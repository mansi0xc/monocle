//! Bounded, single-session browser bridge. Imported JSON never grants trust.
use monocle_core::{
    Code, Diagnostic, EpochCheckpoint, Parsed, TrustBasis, TrustStore, finality::verify_evidence,
    parse_bundle, parse_checkpoint,
};
use serde::Serialize;
use serde_json::{Value, json};

pub const MAX_INPUT_BYTES: usize = monocle_core::MAX_REQUEST_BYTES;

#[derive(Default)]
pub struct Session {
    candidate: Option<Parsed<EpochCheckpoint>>,
    trust: TrustStore,
}

fn diagnostic(code: Code, path: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code,
        path: path.into(),
        message: message.into(),
    }
}

fn value<T: Serialize>(result: Result<T, Diagnostic>) -> Value {
    match result {
        Ok(value) => json!({"ok": true, "value": value}),
        Err(error) => json!({"ok": false, "error": error}),
    }
}

impl Session {
    /// ABI v1 operations: inspect bundle, inspect checkpoint, select synthetic,
    /// select operator (UTF-8 reason), verify request, clear trust.
    pub fn execute(&mut self, operation: u32, input: &[u8]) -> Value {
        // Failed replacement/selection also revokes earlier trust, including
        // inputs rejected by the bridge-wide size check.
        if matches!(operation, 2..=4) {
            self.trust = TrustStore::new();
            if operation == 2 {
                self.candidate = None;
            }
        }
        if input.len() > MAX_INPUT_BYTES {
            return value::<()>(Err(diagnostic(
                Code::InputTooLarge,
                "bridge",
                "Input exceeds the bridge limit",
            )));
        }
        match operation {
            1 => {
                value(parse_bundle(input).map(
                    |parsed| json!({"digest": parsed.digest(), "document": parsed.document()}),
                ))
            }
            2 => value(parse_checkpoint(input).map(|parsed| {
                let result = json!({"digest": parsed.digest(), "document": parsed.document()});
                self.candidate = Some(parsed);
                result
            })),
            3 | 4 => value((|| {
                let checkpoint = self.candidate.clone().ok_or_else(|| {
                    diagnostic(
                        Code::MissingCheckpoint,
                        "checkpoint",
                        "Inspect a checkpoint first",
                    )
                })?;
                let basis = if operation == 3 {
                    if !input.is_empty() {
                        return Err(diagnostic(
                            Code::InvalidSchema,
                            "trust_basis",
                            "Synthetic selection takes no payload",
                        ));
                    }
                    TrustBasis::SyntheticFixture
                } else {
                    let reason = std::str::from_utf8(input).map_err(|_| {
                        diagnostic(
                            Code::InvalidSchema,
                            "trust_basis",
                            "Trust reason must be UTF-8",
                        )
                    })?;
                    TrustBasis::OperatorPinned {
                        reason: reason.to_owned(),
                    }
                };
                self.trust.pin(checkpoint, basis)
            })()),
            5 => value::<_>(Ok(verify_evidence(input, &self.trust))),
            6 => {
                self.candidate = None;
                self.trust = TrustStore::new();
                value::<()>(Ok(()))
            }
            _ => value::<()>(Err(diagnostic(
                Code::InvalidSchema,
                "operation",
                "Unknown bridge operation",
            ))),
        }
    }
}

// The ABI exposes a Rust-owned, bounded input buffer. No raw host pointer is
// accepted or dereferenced. The JavaScript application host itself is trusted.
// Read fresh memory views after execution: allocation may grow WebAssembly
// memory and detach previous JS views.
#[cfg(target_arch = "wasm32")]
mod abi {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    struct Bridge {
        session: Session,
        input: Vec<u8>,
        output: Vec<u8>,
        ready: bool,
    }
    thread_local! { static BRIDGE: RefCell<Bridge> = RefCell::new(Bridge::default()); }

    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_abi_version() -> u32 {
        1
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_input_alloc(length: u32) -> u32 {
        BRIDGE.with_borrow_mut(|bridge| {
            bridge.ready = false;
            bridge.output.clear();
            bridge.input.clear();
            let length = length as usize;
            if length > MAX_INPUT_BYTES || bridge.input.try_reserve_exact(length).is_err() {
                bridge.session = Session::default();
                return 0;
            }
            bridge.input.resize(length, 0);
            bridge.ready = true;
            bridge.input.as_mut_ptr() as u32
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_execute(operation: u32) -> u32 {
        BRIDGE.with_borrow_mut(|bridge| {
            bridge.output.clear();
            if !bridge.ready {
                return 0;
            }
            bridge.ready = false;
            let response = bridge.session.execute(operation, &bridge.input);
            bridge.input.clear();
            match serde_json::to_vec(&response) {
                Ok(bytes) => {
                    bridge.output = bytes;
                    1
                }
                Err(_) => 0,
            }
        })
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_output_ptr() -> u32 {
        BRIDGE.with_borrow(|bridge| bridge.output.as_ptr() as u32)
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn monocle_output_len() -> u32 {
        BRIDGE.with_borrow(|bridge| bridge.output.len() as u32)
    }
}
