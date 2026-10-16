//! monocle-genesis-checkpoint <validators-capture> <out-dir> [genesis-cert-capture]
//!
//! Writes checkpoint.json and summary.json, plus bundle.json when a recorded
//! getAgGenesisCert capture is given. Reads only local files.
use monocle_genesis_checkpoint::{build_bundle, build_checkpoint, read_capture};
use std::{fs, path::PathBuf, process::ExitCode};

fn run() -> Result<(), String> {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [validators, out, rest @ ..] = args.as_slice() else {
        return Err(
            "usage: monocle-genesis-checkpoint <validators-capture> <out-dir> [genesis-cert-capture]"
                .into(),
        );
    };
    if rest.len() > 1 {
        return Err("expected at most one genesis-cert capture".into());
    }
    let checkpoint = build_checkpoint(&read_capture(validators, "validators")?)?;
    fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let write = |name: &str, text: &str| {
        fs::write(out.join(name), text).map_err(|e| format!("{name}: {e}"))
    };
    write("checkpoint.json", &checkpoint.json)?;
    let summary = serde_json::to_string_pretty(&checkpoint.summary).map_err(|e| e.to_string())?;
    write("summary.json", &(summary + "\n"))?;
    if let Some(cert) = rest.first() {
        write(
            "bundle.json",
            &build_bundle(&read_capture(cert, "genesis-cert")?, &checkpoint)?,
        )?;
    }
    eprintln!(
        "Rank map for epoch {}: {} validators, total stake {}. {}",
        checkpoint.context.epoch,
        checkpoint.summary["ranked"],
        checkpoint.summary["total_stake"],
        if rest.is_empty() {
            "No certificate bundle requested."
        } else {
            "Bundle written."
        }
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}
