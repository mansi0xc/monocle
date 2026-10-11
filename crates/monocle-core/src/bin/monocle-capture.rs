//! Bounded local-file/stdin inspection. No network, trust selection, or file writes.
use monocle_core::capture::{MAX_CAPTURE_BYTES, inspect_genesis_rpc};
use std::{
    fs::File,
    io::{self, Read, Write},
    process::ExitCode,
};
fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("Usage: monocle-capture <genesis-rpc.json | ->".into());
    }
    let input: Box<dyn Read> = if args[0] == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(&args[0]).map_err(|e| format!("Could not open capture: {e}"))?)
    };
    let mut bytes = Vec::new();
    input
        .take((MAX_CAPTURE_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("Could not read capture: {e}"))?;
    let report = inspect_genesis_rpc(&bytes).map_err(|e| {
        serde_json::to_string(&e).unwrap_or_else(|_| "Capture parsing failed".into())
    })?;
    let output = serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?;
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&output)
        .and_then(|_| stdout.write_all(b"\n"))
        .map_err(|e| e.to_string())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
