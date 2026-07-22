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
    let report_json: Value = serde_json::from_str(report).map_err(|_| invalid())?;
    if !has_runtime_chain(&report_json)? {
        return Ok(());
    }
    let paths = actor_paths(&report_json)?;
    let chain = super::runtime_receipt_chain::build_runtime_receipt_chain_from_actor_paths(&paths)
        .map_err(super::independent_verifier_errors::map_actor_verification_error)?;
    require_settlement_commitment(&report_json, chain.as_str())?;
    super::three_agent_transcript::require_runtime_chain_source(report, chain.as_str())
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
    let has_actor_sources = ACTOR_FILES
        .iter()
        .any(|(field, _)| !report["artifacts"][*field].is_null());
    if !has_actor_sources {
        return Ok(false);
    }
    let Some(path) = report["artifacts"]["three_agent_transcript"].as_str() else {
        return Err(invalid());
    };
    let raw = std::fs::read_to_string(path).map_err(|_| invalid())?;
    if raw.contains("kamn.service.receipt-chain.v1") {
        return Ok(true);
    }
    Err(invalid())
}

fn require_settlement_commitment(report: &Value, chain: &str) -> Result<(), String> {
    let path = report["artifacts"]["devnet_settlement_evidence"]
        .as_str()
        .ok_or_else(invalid)?;
    let evidence =
        super::settlement_evidence_artifact::read_settlement_evidence_artifact(Path::new(path))?;
    let chain: Value = serde_json::from_str(chain).map_err(|_| invalid())?;
    let expected = chain["receipt_chain_commitment"].as_str();
    if evidence.receipt_chain_commitment.as_deref() == expected {
        return Ok(());
    }
    Err(invalid())
}

fn invalid() -> String {
    "RECEIPT_CHAIN_INVALID".to_owned()
}
