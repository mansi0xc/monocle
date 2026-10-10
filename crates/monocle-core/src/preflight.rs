use crate::*;

const STAGES: [Stage; 9] = [
    Stage::Input,
    Stage::Profile,
    Stage::Context,
    Stage::CertificateDecoding,
    Stage::Signers,
    Stage::KeyQualification,
    Stage::Signature,
    Stage::Quorum,
    Stage::Finality,
];
impl VerificationReport {
    fn empty() -> Self {
        Self {
            schema_version: 1,
            run_id: None,
            bundle_digest: None,
            profile_id: None,
            checkpoint_digest: None,
            conclusion: Conclusion::Inconclusive,
            evidence_scope: "preflight_only",
            assumptions: Vec::new(),
            checks: STAGES
                .into_iter()
                .map(|stage| CheckResult {
                    stage,
                    status: CheckStatus::NotChecked,
                    diagnostic: Diagnostic::new(
                        Code::PrerequisiteMissing,
                        "$",
                        "Earlier prerequisites have not passed",
                    ),
                })
                .collect(),
        }
    }
    fn check(&mut self, stage: Stage, status: CheckStatus, diagnostic: Diagnostic) {
        if let Some(check) = self.checks.iter_mut().find(|check| check.stage == stage) {
            check.status = status;
            check.diagnostic = diagnostic;
        }
    }
    fn pass(&mut self, stage: Stage, message: &str) {
        self.check(
            stage,
            CheckStatus::Pass,
            Diagnostic::new(Code::Ok, "$", message),
        );
    }
    fn stop(mut self, stage: Stage, error: Diagnostic, malformed: bool) -> Self {
        self.conclusion = if malformed {
            Conclusion::Rejected
        } else {
            Conclusion::Inconclusive
        };
        self.check(
            stage,
            if malformed {
                CheckStatus::Fail
            } else {
                CheckStatus::NotChecked
            },
            error,
        );
        self
    }
}

/// Parse and resolve local context only. This function can NEVER return VERIFIED.
/// No request or bundle field is allowed to create a trusted checkpoint.
pub fn preflight(request_bytes: &[u8], trust: &TrustStore) -> VerificationReport {
    let mut report = VerificationReport::empty();
    let request = match parse_request(request_bytes) {
        Ok(request) => request,
        Err(error) => {
            let malformed = error.code != Code::UnsupportedVersion;
            return report.stop(Stage::Input, error, malformed);
        }
    };
    let request = request.document();
    report.run_id = Some(request.run_id.clone());
    report.profile_id = Some(request.profile_id.clone());
    report.checkpoint_digest = request.selected_checkpoint.clone();
    report.bundle_digest = Some(ContentDigest::of(request.bundle_json.as_bytes()));
    let bundle = match parse_bundle(request.bundle_json.as_bytes()) {
        Ok(bundle) => bundle,
        Err(error) => {
            let malformed = error.code != Code::UnsupportedVersion;
            return report.stop(Stage::Input, error, malformed);
        }
    };
    let bundle = bundle.document();
    report.pass(
        Stage::Input,
        "Bounded schema and transport checks passed; certificate bytes remain unverified",
    );
    let Some(profile) = profile_by_id(&request.profile_id) else {
        return report.stop(
            Stage::Profile,
            Diagnostic::new(
                Code::UnsupportedProfile,
                "profile_id",
                "Selected profile is not built into this core",
            ),
            false,
        );
    };
    if bundle.profile_id != request.profile_id {
        return report.stop(
            Stage::Profile,
            Diagnostic::new(
                Code::ProfileMismatch,
                "profile_id",
                "Bundle profile differs from the selected profile",
            ),
            false,
        );
    }
    if !profile
        .recognized_claim_kinds
        .contains(&bundle.claim.kind.as_str())
        || bundle.certificates.iter().any(|cert| {
            !profile.recognized_formats.contains(&cert.format.as_str())
                || !profile
                    .recognized_certificate_kinds
                    .contains(&cert.kind.as_str())
        })
    {
        return report.stop(
            Stage::Profile,
            Diagnostic::new(
                Code::UnsupportedCertificate,
                "certificates",
                "Unrecognized claim, certificate kind, or source format",
            ),
            false,
        );
    }
    report.pass(
        Stage::Profile,
        "Recognized profile metadata; no certificate verification is performed by preflight",
    );
    let Some(digest) = request.selected_checkpoint.as_ref() else {
        return report.stop(
            Stage::Context,
            Diagnostic::new(
                Code::MissingCheckpoint,
                "selected_checkpoint",
                "Select an independently trusted checkpoint",
            ),
            false,
        );
    };
    let Some(checkpoint) = trust.get(digest) else {
        return report.stop(
            Stage::Context,
            Diagnostic::new(
                Code::CheckpointNotTrusted,
                "selected_checkpoint",
                "Selected digest is absent from the local trust store",
            ),
            false,
        );
    };
    if bundle
        .checkpoint_reference
        .as_ref()
        .is_some_and(|reference| reference != digest)
    {
        return report.stop(
            Stage::Context,
            Diagnostic::new(
                Code::CheckpointMismatch,
                "checkpoint_reference",
                "Bundle reference differs from the explicitly selected checkpoint",
            ),
            false,
        );
    }
    let data = checkpoint.parsed.document();
    if data.profile_id != profile.id || data.context != bundle.context {
        return report.stop(
            Stage::Context,
            Diagnostic::new(
                Code::ContextMismatch,
                "context",
                "Checkpoint profile, genesis hash, epoch, or shred version differs",
            ),
            false,
        );
    }
    if matches!(checkpoint.basis, TrustBasis::SyntheticFixture)
        && bundle.provenance.kind != SourceKind::Synthetic
    {
        return report.stop(
            Stage::Context,
            Diagnostic::new(
                Code::ContextMismatch,
                "provenance.kind",
                "Synthetic fixture trust cannot support a captured-network claim",
            ),
            false,
        );
    }
    report.pass(Stage::Context, "Explicit local selection matches declared context; authenticity remains the caller's assumption");
    report.assumptions.push(match &checkpoint.basis {
        TrustBasis::SyntheticFixture => {
            "Local synthetic fixture selection; no real validator-set authenticity claim".into()
        }
        TrustBasis::OperatorPinned { reason } => format!("Operator-pinned checkpoint: {reason}"),
    });
    report.assumptions.push(
        "Trust selection alone does not qualify BLS keys; later checks must validate each selected signer's PoP"
            .into(),
    );
    for stage in STAGES.into_iter().skip(3) {
        report.check(
            stage,
            CheckStatus::NotChecked,
            Diagnostic::new(
                Code::VerifierNotImplemented,
                "$",
                "This preflight call does not perform certificate verification or finality evaluation",
            ),
        );
    }
    report
}
