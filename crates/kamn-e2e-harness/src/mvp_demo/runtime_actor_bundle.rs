use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) const ACTOR_FILES: [(&str, &str); 3] = [
    ("runtime_agent_a_evidence", "runtime-agent-a-evidence.json"),
    ("runtime_agent_b_evidence", "runtime-agent-b-evidence.json"),
    ("runtime_agent_c_evidence", "runtime-agent-c-evidence.json"),
];

pub(super) fn copy_runtime_actor_sources(
    paths: Option<&[String; 3]>,
    run_dir: &Path,
) -> Result<(), String> {
    let Some(paths) = paths else {
        return Ok(());
    };
    super::runtime_receipt_chain::build_runtime_receipt_chain_from_actor_paths(paths)?;
    for (source, (_, file)) in paths.iter().zip(ACTOR_FILES) {
        copy_source(Path::new(source), run_dir.join("proof").join(file))?;
    }
    Ok(())
}

pub(super) fn validate_runtime_actor_bundle(report: &str) -> Result<(), String> {
    let report: Value = serde_json::from_str(report).map_err(|_| invalid())?;
    if !has_runtime_chain(&report)? {
        return Ok(());
    }
    let paths = actor_paths(&report)?;
    let chain = super::runtime_receipt_chain::build_runtime_receipt_chain_from_actor_paths(&paths)
        .map_err(super::independent_verifier_errors::map_actor_verification_error)?;
    super::three_agent_transcript::require_runtime_chain_source(
        serde_json::to_string(&report)
            .map_err(|_| invalid())?
            .as_str(),
        chain.as_str(),
    )
    .map_err(super::independent_verifier_errors::map_actor_verification_error)
}

pub(super) fn report_entries(run_dir: &Path) -> Vec<(&'static str, String)> {
    ACTOR_FILES
        .into_iter()
        .map(|(field, file)| (field, run_dir.join("proof").join(file)))
        .filter(|(_, path)| path.is_file())
        .map(|(field, path)| (field, path.display().to_string()))
        .collect()
}

fn actor_paths(report: &Value) -> Result<[String; 3], String> {
    let values =
        ACTOR_FILES.map(|(field, _)| report["artifacts"][field].as_str().map(str::to_owned));
    match values {
        [Some(a), Some(b), Some(c)] => Ok([a, b, c]),
        _ => Err(invalid()),
    }
}

fn copy_source(source: &Path, destination: PathBuf) -> Result<(), String> {
    std::fs::copy(source, destination)
        .map(|_| ())
        .map_err(|_| invalid())
}

fn has_runtime_chain(report: &Value) -> Result<bool, String> {
    let Some(path) = report["artifacts"]["three_agent_transcript"].as_str() else {
        return Ok(false);
    };
    let raw = std::fs::read_to_string(path).map_err(|_| invalid())?;
    Ok(raw.contains("kamn.mvp.runtime-receipt-chain.v1"))
}

fn invalid() -> String {
    "RECEIPT_CHAIN_INVALID".to_owned()
}
