use monocle_core::{certificate, finality::*, *};
use serde_json::{Value, json};

fn corpus() -> Value {
    serde_json::from_str(include_str!("../../../fixtures/finality/corpus.json")).unwrap()
}
fn fixture(name: &str) -> Value {
    corpus()["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap()
        .clone()
}
fn prepared(case: &Value) -> (Vec<u8>, TrustStore, ContentDigest) {
    let mut trust = TrustStore::new();
    let digest = trust
        .pin(
            parse_checkpoint(case["checkpoint_json"].as_str().unwrap().as_bytes()).unwrap(),
            TrustBasis::SyntheticFixture,
        )
        .unwrap();
    let bundle: Value = serde_json::from_str(case["bundle_json"].as_str().unwrap()).unwrap();
    let request = VerifyRequest {
        schema_version: 1,
        run_id: "test".into(),
        profile_id: bundle["profile_id"].as_str().unwrap().into(),
        bundle_json: case["bundle_json"].as_str().unwrap().into(),
        selected_checkpoint: case["select_checkpoint"]
            .as_bool()
            .unwrap()
            .then_some(digest.clone()),
    };
    (serde_json::to_vec(&request).unwrap(), trust, digest)
}
fn run(case: &Value) -> EvidenceVerificationReport {
    let (req, trust, _) = prepared(case);
    verify_evidence(&req, &trust)
}
fn change_bundle(case: &mut Value, mutate: impl FnOnce(&mut Value)) {
    let mut bundle: Value = serde_json::from_str(case["bundle_json"].as_str().unwrap()).unwrap();
    mutate(&mut bundle);
    case["bundle_json"] = json!(serde_json::to_string(&bundle).unwrap());
}

#[test]
fn complete_signed_corpus_has_expected_scopes_paths_and_diagnostics() {
    let corpus = corpus();
    assert_eq!(corpus["cases"].as_array().unwrap().len(), 35);
    for case in corpus["cases"].as_array().unwrap() {
        let report = serde_json::to_value(run(case)).unwrap();
        for field in [
            "conclusion",
            "evidence_scope",
            "finality_path",
            "supporting_certificate_ids",
        ] {
            assert_eq!(
                report[field], case["expected"][field],
                "{} {field}",
                case["name"]
            );
        }
        assert!(
            report["checks"]
                .as_array()
                .unwrap()
                .iter()
                .chain(
                    report["certificates"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .flat_map(|c| c["checks"].as_array().unwrap())
                )
                .any(|c| c["diagnostic"]["code"] == case["expected"]["diagnostic"]),
            "{}",
            case["name"]
        );
        assert_eq!(
            report["checks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|c| c["stage"] == "finality")
                .count(),
            1
        );
        if report["conclusion"] != "VERIFIED" {
            assert_eq!(report["evidence_scope"], "unestablished");
            assert!(report["finality_path"].is_null());
            assert_eq!(report["supporting_certificate_ids"], json!([]));
        }
    }
}

#[test]
fn composition_preserves_every_individual_report_and_c3_stays_scoped() {
    for case in corpus()["cases"].as_array().unwrap() {
        let (request, trust, _) = prepared(case);
        let c3 = certificate::verify_certificates(&request, &trust);
        let c4 = verify_evidence(&request, &trust);
        assert_eq!(
            serde_json::to_value(&c3.certificates).unwrap(),
            serde_json::to_value(&c4.certificates).unwrap()
        );
        assert_eq!(c3.bundle_digest, c4.bundle_digest);
        assert_eq!(c3.checkpoint_digest, c4.checkpoint_digest);
        if c4.evidence_scope == EvidenceScope::BlockFinality
            || c4.evidence_scope == EvidenceScope::GenesisMigration
        {
            assert_eq!(c3.conclusion, Conclusion::Inconclusive);
            assert_eq!(c3.checks.last().unwrap().status, CheckStatus::NotChecked);
        }
    }
}

#[test]
fn slow_path_keeps_different_signer_sets_and_slot_only_finalize_payload() {
    let report = run(&fixture("slow_different_signers"));
    assert_eq!(report.finality_path, Some(FinalityPath::Slow));
    let n = &report.certificates[0];
    let f = &report.certificates[1];
    assert_eq!(n.selected_stake.unwrap().get(), 65);
    assert_eq!(f.selected_stake.unwrap().get(), 75);
    assert_ne!(n.signer_ranks, f.signer_ranks);
    assert!(f.block_id_hex.is_none());
    assert_eq!(f.signed_payload_hex.as_ref().unwrap().len(), 22);
    assert_eq!(n.signed_payload_hex.as_ref().unwrap().len(), 86);
    assert_eq!(report.evidence_scope, EvidenceScope::BlockFinality);
}

#[test]
fn witness_selection_is_deterministic_without_discarding_extra_checks() {
    let mut case = fixture("fast_and_slow");
    change_bundle(&mut case, |b| {
        let mut fast = b["certificates"][2].clone();
        fast["id"] = json!("aaa-fast");
        b["certificates"].as_array_mut().unwrap().push(fast);
        b["claim"]["certificate_ids"]
            .as_array_mut()
            .unwrap()
            .push(json!("aaa-fast"));
    });
    let first = run(&case);
    change_bundle(&mut case, |b| {
        b["certificates"].as_array_mut().unwrap().reverse();
        b["claim"]["certificate_ids"]
            .as_array_mut()
            .unwrap()
            .reverse();
    });
    let second = run(&case);
    assert_eq!(first.supporting_certificate_ids, vec!["aaa-fast"]);
    assert_eq!(
        first.supporting_certificate_ids,
        second.supporting_certificate_ids
    );
    assert_eq!(first.finality_path, second.finality_path);
    assert_eq!(second.certificates.len(), 4);
    assert!(
        second
            .certificates
            .iter()
            .all(|c| c.conclusion == Conclusion::Verified)
    );
}

#[test]
fn separate_under_quorum_certificates_cannot_be_added_into_a_quorum() {
    let mut case = fixture("notar_under_quorum");
    change_bundle(&mut case, |b| {
        let mut duplicate = b["certificates"][0].clone();
        duplicate["id"] = json!("second-notar");
        b["certificates"].as_array_mut().unwrap().push(duplicate);
        b["claim"]["certificate_ids"]
            .as_array_mut()
            .unwrap()
            .push(json!("second-notar"));
    });
    let report = run(&case);
    assert_eq!(report.conclusion, Conclusion::Rejected);
    assert_eq!(report.evidence_scope, EvidenceScope::Unestablished);
    for index in [0, 2] {
        let checks = &report.certificates[index].checks;
        assert_eq!(
            checks
                .iter()
                .find(|c| c.stage == Stage::Signature)
                .unwrap()
                .status,
            CheckStatus::Pass
        );
        assert_eq!(
            checks
                .iter()
                .find(|c| c.stage == Stage::Quorum)
                .unwrap()
                .diagnostic
                .code,
            Code::InsufficientStake
        );
    }
}

#[test]
fn each_run_resolves_trust_again_and_cannot_reuse_a_previous_success() {
    let (request, mut trust, digest) = prepared(&fixture("fast_finality"));
    assert_eq!(
        verify_evidence(&request, &trust).conclusion,
        Conclusion::Verified
    );
    trust.remove(&digest);
    let report = verify_evidence(&request, &trust);
    assert_eq!(report.conclusion, Conclusion::Inconclusive);
    assert_eq!(report.evidence_scope, EvidenceScope::Unestablished);
    assert!(report.certificates.is_empty());
    assert_eq!(report.checks[2].diagnostic.code, Code::CheckpointNotTrusted);
}

#[test]
fn imported_success_and_policy_fields_are_not_authoritative() {
    for field in [
        "report",
        "evidence_scope",
        "policy_id",
        "supporting_certificate_ids",
    ] {
        let mut case = fixture("fast_finality");
        change_bundle(&mut case, |b| b[field] = json!("VERIFIED"));
        let report = run(&case);
        assert_eq!(report.conclusion, Conclusion::Rejected);
        assert_eq!(report.checks[0].diagnostic.code, Code::UnknownField);
        assert_eq!(report.evidence_scope, EvidenceScope::Unestablished);
    }
}

#[test]
fn wrong_epoch_cluster_and_shred_context_cannot_be_composed() {
    for (field, value) in [
        ("epoch", json!("43")),
        ("genesis_hash_hex", json!("22".repeat(32))),
        ("shred_version", json!(1235)),
    ] {
        let mut case = fixture("slow_finality");
        change_bundle(&mut case, |b| b["context"][field] = value);
        let report = run(&case);
        assert_eq!(report.conclusion, Conclusion::Inconclusive);
        assert_eq!(report.checks[2].diagnostic.code, Code::ContextMismatch);
        assert_eq!(report.evidence_scope, EvidenceScope::Unestablished);
    }
}

#[test]
fn genesis_proves_an_anchor_scope_without_ordinary_finality_or_transition_claims() {
    let report = run(&fixture("genesis_anchor"));
    assert_eq!(report.evidence_scope, EvidenceScope::GenesisMigration);
    assert!(report.finality_path.is_none());
    assert_eq!(report.supporting_certificate_ids, vec!["genesis"]);
    assert!(
        report
            .assumptions
            .iter()
            .any(|a| a.contains("does not prove that a node or the cluster completed migration"))
    );
    assert_eq!(
        run(&fixture("genesis_as_block")).conclusion,
        Conclusion::Rejected
    );
}

#[test]
fn cryptographic_failure_does_not_get_mislabeled_as_missing_counterpart() {
    for name in [
        "notar_under_quorum",
        "finalize_under_quorum",
        "changed_signature",
    ] {
        let report = run(&fixture(name));
        let finality = report.checks.last().unwrap();
        assert_eq!(finality.status, CheckStatus::NotChecked);
        assert_eq!(finality.diagnostic.code, Code::CertificateChecksFailed);
        assert_eq!(report.conclusion, Conclusion::Rejected);
    }
    for name in ["notar_only", "finalize_only"] {
        let report = run(&fixture(name));
        assert_eq!(report.conclusion, Conclusion::Inconclusive);
        assert_eq!(report.certificates[0].conclusion, Conclusion::Verified);
    }
}
