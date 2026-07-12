use serde_json::Value;
use std::path::{Component, Path, PathBuf};

use super::settlement_evidence_artifact::{
    read_settlement_evidence_artifact, SettlementEvidenceArtifact,
};

const PATH_INVALID: &str = "PROOF_ARTIFACT_PATH_INVALID";
const SETTLEMENT_INVALID: &str = "SETTLEMENT_EVIDENCE_INVALID";
const AGENT_CLAIM_INVALID: &str = "AGENT_TRANSACTION_CLAIM_INVALID";
pub(super) fn validate_independent_bundle(report: &str, report_path: &str) -> Result<(), String> {
    let report_json: Value =
        serde_json::from_str(report).map_err(|_| "PROOF_ARTIFACT_TAMPERED".to_owned())?;
    let context = BundleContext::new(&report_json, report_path)?;
    let has_claim = has_agent_transaction(&report_json);
    if context.has_agent_transaction_files() && !has_claim {
        return Err(AGENT_CLAIM_INVALID.to_owned());
    }
    if !has_claim {
        return Ok(());
    }
    context.validate_paths()?;
    let evidence = context.read_settlement_evidence()?;
    super::independent_settlement_verify::validate_settlement(&report_json, &evidence)?;
    super::settlement_log_verify::validate_settlement_log(&report_json, &evidence)?;
    super::authoritative_rpc_verify::validate_authoritative_rpc(&report_json, &evidence)?;
    super::independent_explorer_verify::validate_explorer_links(context.markdown_paths(), &evidence)
}

struct BundleContext<'a> {
    report: &'a Value,
    output_root: PathBuf,
    run_dir: PathBuf,
    report_path: PathBuf,
}

impl<'a> BundleContext<'a> {
    fn new(report: &'a Value, report_path: &str) -> Result<Self, String> {
        let output_root = output_root(Path::new(report_path))?;
        let run_id = string(report, "run_id").ok_or_else(path_invalid)?;
        validate_run_id(run_id)?;
        let run_dir = output_root.join(run_id);
        Ok(Self {
            report,
            output_root,
            run_dir,
            report_path: PathBuf::from(report_path),
        })
    }

    fn validate_paths(&self) -> Result<(), String> {
        self.validate_report_paths()?;
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

    fn validate_report_paths(&self) -> Result<(), String> {
        let indexed_json = artifact_path(self.report, "report_json")?;
        require_same_file(
            Path::new(indexed_json),
            self.run_dir.join("proof/report.json").as_path(),
        )?;
        let indexed_md = artifact_path(self.report, "report_md")?;
        require_same_file(
            Path::new(indexed_md),
            self.run_dir.join("proof/report.md").as_path(),
        )?;
        validate_supplied_report_path(self)
    }

    fn read_settlement_evidence(&self) -> Result<SettlementEvidenceArtifact, String> {
        let path = artifact_path(self.report, "devnet_settlement_evidence")?;
        read_settlement_evidence_artifact(Path::new(path))
    }

    fn markdown_paths(&self) -> [PathBuf; 2] {
        [
            self.run_dir.join("proof/report.md"),
            self.report_path.with_file_name("report.md"),
        ]
    }

    fn has_agent_transaction_files(&self) -> bool {
        [
            "three-agent-transcript.json",
            "live-task-settlement-binding.json",
            "runtime-agent-a-evidence.json",
        ]
        .iter()
        .any(|name| self.run_dir.join("proof").join(name).is_file())
    }
}

fn validate_supplied_report_path(context: &BundleContext<'_>) -> Result<(), String> {
    let supplied = context
        .report_path
        .canonicalize()
        .map_err(|_| path_invalid())?;
    let concrete = context
        .run_dir
        .join("proof/report.json")
        .canonicalize()
        .map_err(|_| path_invalid())?;
    let latest = context
        .output_root
        .join("latest/proof/report.json")
        .canonicalize()
        .map_err(|_| path_invalid())?;
    if supplied == concrete || supplied == latest {
        return Ok(());
    }
    Err(path_invalid())
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
    if !path.exists() {
        return Err("PROOF_ARTIFACT_MISSING".to_owned());
    }
    let canonical = path.canonicalize().map_err(|_| path_invalid())?;
    let root = run_dir.canonicalize().map_err(|_| path_invalid())?;
    if canonical.starts_with(root) {
        return Ok(());
    }
    Err(path_invalid())
}

fn require_same_file(actual: &Path, expected: &Path) -> Result<(), String> {
    let actual = actual.canonicalize().map_err(|_| path_invalid())?;
    let expected = expected.canonicalize().map_err(|_| path_invalid())?;
    if actual == expected {
        return Ok(());
    }
    Err(path_invalid())
}

fn validate_run_id(run_id: &str) -> Result<(), String> {
    let mut components = Path::new(run_id).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(_)), None) => Ok(()),
        _ => Err(path_invalid()),
    }
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
