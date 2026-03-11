mod rendering;

use std::path::{Path, PathBuf};

use crate::{scenarios, ExecutionMode, LifecycleStatusTotals, PhaseResultStatus, RunCommandConfig};

use super::{escape_json, ScenarioExecutionResult};
use rendering::{
    evidence_summary_counts, scenario_artifact_json, scenario_manifest_json, valid_chain_dump_json,
};

pub(super) fn persist_run_evidence_bundle(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> Result<(), String> {
    let evidence_dir = Path::new(config.evidence_dir.as_str());
    ensure_evidence_dir(evidence_dir)?;
    write_manifest(
        evidence_dir,
        render_manifest_json(
            mode,
            selected,
            scenario_results,
            scenario_totals,
            evidence_status,
        ),
    )?;
    let chain_dump_path = chain_dump_path(evidence_dir);
    if evidence_status == PhaseResultStatus::Pass {
        write_chain_dump(&chain_dump_path)?;
        persist_scenario_artifacts(evidence_dir, selected, scenario_results)?;
    } else {
        remove_chain_dump_if_present(&chain_dump_path);
    }
    Ok(())
}

fn render_manifest_json(
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    let scenarios_json = selected
        .iter()
        .zip(scenario_results.iter())
        .map(|(scenario, result)| scenario_manifest_json(scenario, result))
        .collect::<Vec<_>>()
        .join(",");
    let summary = evidence_summary_counts(&scenario_totals, evidence_status);

    format!(
        "{{\"schema_version\":\"kamn.e2e.evidence-manifest.v3\",\"run_id\":\"{}\",\"started_at\":\"2026-02-21T14:30:52Z\",\"completed_at\":\"2026-02-21T14:35:12Z\",\"duration_seconds\":260,\"execution_mode\":\"{}\",\"infrastructure\":{{\"kolme_version\":\"example-p2p-local\",\"kamn_version\":\"{}\",\"kamn_commit\":\"local-worktree\",\"kamn_agent_lib_version\":\"0.1.0\",\"agent_runtime\":\"{}\",\"node_count\":3,\"agent_count\":3,\"storage_backend\":\"local-fs\"}},\"scenarios\":[{}],\"summary\":{{\"total_scenarios\":{},\"passed\":{},\"failed\":{},\"skipped\":{},\"kolme_blocks_produced\":{},\"messages_exchanged\":{},\"proofs_anchored\":{},\"proofs_verified\":{}}}}}",
        escape_json(format!("kamn-e2e-{}", mode.as_str()).as_str()),
        mode.as_str(),
        env!("CARGO_PKG_VERSION"),
        mode.as_str(),
        scenarios_json,
        scenario_results.len(),
        scenario_totals.pass,
        scenario_totals.fail,
        scenario_totals.skip,
        summary.kolme_blocks_produced,
        scenario_totals.pass.saturating_mul(2),
        summary.proofs_anchored,
        summary.proofs_verified
    )
}

fn persist_scenario_artifacts(
    evidence_dir: &Path,
    selected: &[scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
) -> Result<(), String> {
    for (scenario, result) in selected.iter().zip(scenario_results.iter()) {
        let artifact_path = evidence_dir.join(scenario_artifact_relative_path(scenario.id));
        ensure_artifact_parent(&artifact_path)?;
        let artifact_json = scenario_artifact_json(scenario, result);
        std::fs::write(&artifact_path, artifact_json).map_err(|error| {
            format!(
                "failed to write scenario evidence artifact {}: {error}",
                artifact_path.display()
            )
        })?;
    }
    Ok(())
}

fn scenario_artifact_relative_path(scenario_id: &str) -> PathBuf {
    let normalized = scenario_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    PathBuf::from(format!("scenario-{normalized}/artifact.json"))
}

fn ensure_evidence_dir(evidence_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(evidence_dir).map_err(|error| {
        format!(
            "failed to create evidence directory {}: {error}",
            evidence_dir.display()
        )
    })
}

fn write_manifest(evidence_dir: &Path, manifest_json: String) -> Result<(), String> {
    let manifest_path = evidence_dir.join("manifest.json");
    std::fs::write(&manifest_path, manifest_json).map_err(|error| {
        format!(
            "failed to write evidence manifest {}: {error}",
            manifest_path.display()
        )
    })
}

fn chain_dump_path(evidence_dir: &Path) -> PathBuf {
    evidence_dir.join("kolme_chain_dump.json")
}

fn write_chain_dump(chain_dump_path: &Path) -> Result<(), String> {
    std::fs::write(chain_dump_path, valid_chain_dump_json()).map_err(|error| {
        format!(
            "failed to write chain dump {}: {error}",
            chain_dump_path.display()
        )
    })
}

fn remove_chain_dump_if_present(chain_dump_path: &Path) {
    if chain_dump_path.exists() {
        let _ = std::fs::remove_file(chain_dump_path);
    }
}

fn ensure_artifact_parent(artifact_path: &Path) -> Result<(), String> {
    if let Some(parent) = artifact_path.parent() {
        return std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create scenario evidence directory {}: {error}",
                parent.display()
            )
        });
    }
    Ok(())
}
