use serde_json::Value;
use std::path::{Path, PathBuf};

use super::settlement_evidence_artifact::{
    read_settlement_evidence_artifact, SettlementEvidenceArtifact,
};

const PATH_INVALID: &str = "PROOF_ARTIFACT_PATH_INVALID";
const SETTLEMENT_INVALID: &str = "SETTLEMENT_EVIDENCE_INVALID";
const EXPLORER_INVALID: &str = "EXPLORER_LINK_INVALID";
pub(super) fn validate_independent_bundle(report: &str, report_path: &str) -> Result<(), String> {
    let report_json: Value =
        serde_json::from_str(report).map_err(|_| "PROOF_ARTIFACT_TAMPERED".to_owned())?;
    if !has_agent_transaction(&report_json) {
        return Ok(());
    }
    let context = BundleContext::new(&report_json, report_path)?;
    context.validate_paths()?;
    let evidence = context.read_settlement_evidence()?;
    super::independent_settlement_verify::validate_settlement(&report_json, &evidence)?;
    super::settlement_log_verify::validate_settlement_log(&report_json, &evidence)?;
    validate_explorer_link(&context, &evidence)
}

struct BundleContext<'a> {
    report: &'a Value,
    output_root: PathBuf,
    run_dir: PathBuf,
}

impl<'a> BundleContext<'a> {
    fn new(report: &'a Value, report_path: &str) -> Result<Self, String> {
        let output_root = output_root(Path::new(report_path))?;
        let run_id = string(report, "run_id").ok_or_else(path_invalid)?;
        let run_dir = output_root.join(run_id);
        Ok(Self {
            report,
            output_root,
            run_dir,
        })
    }

    fn validate_paths(&self) -> Result<(), String> {
        let artifacts = self.report["artifacts"]
            .as_object()
            .ok_or_else(path_invalid)?;
        for (name, value) in artifacts {
            if name == "report_json" || name == "report_md" {
                continue;
            }
            let path = value.as_str().ok_or_else(path_invalid)?;
            require_contained(Path::new(path), self.run_dir.as_path())?;
        }
        Ok(())
    }

    fn read_settlement_evidence(&self) -> Result<SettlementEvidenceArtifact, String> {
        let path = artifact_path(self.report, "devnet_settlement_evidence")?;
        read_settlement_evidence_artifact(Path::new(path))
    }

    fn markdown_path(&self) -> PathBuf {
        self.output_root.join("latest/proof/report.md")
    }
}

fn validate_explorer_link(
    context: &BundleContext<'_>,
    evidence: &SettlementEvidenceArtifact,
) -> Result<(), String> {
    let markdown =
        std::fs::read_to_string(context.markdown_path()).map_err(|_| explorer_invalid())?;
    let expected = format!(
        "https://explorer.solana.com/tx/{}?cluster=devnet",
        evidence.settlement_tx_signature
    );
    if markdown.contains(expected.as_str()) {
        return Ok(());
    }
    Err(explorer_invalid())
}

fn output_root(report_path: &Path) -> Result<PathBuf, String> {
    report_path
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(path_invalid)
}

fn require_contained(path: &Path, run_dir: &Path) -> Result<(), String> {
    let canonical = path.canonicalize().map_err(|_| path_invalid())?;
    let root = run_dir.canonicalize().map_err(|_| path_invalid())?;
    if canonical.starts_with(root) {
        return Ok(());
    }
    Err(path_invalid())
}

fn has_agent_transaction(report: &Value) -> bool {
    claim(report, "three_agent_escrow_verification").is_ok()
}

fn claim<'a>(report: &'a Value, id: &str) -> Result<&'a Value, String> {
    report["claim_matrix"]
        .as_array()
        .and_then(|claims| claims.iter().find(|claim| claim["id"] == id))
        .ok_or_else(settlement_invalid)
}

fn artifact_path<'a>(report: &'a Value, name: &str) -> Result<&'a str, String> {
    report["artifacts"][name]
        .as_str()
        .ok_or_else(settlement_invalid)
}

fn string<'a>(value: &'a Value, field: &str) -> Option<&'a str> {
    value[field].as_str()
}

fn path_invalid() -> String {
    PATH_INVALID.to_owned()
}

fn settlement_invalid() -> String {
    SETTLEMENT_INVALID.to_owned()
}

fn explorer_invalid() -> String {
    EXPLORER_INVALID.to_owned()
}
