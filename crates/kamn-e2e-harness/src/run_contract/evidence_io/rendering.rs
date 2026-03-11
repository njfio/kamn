use std::path::PathBuf;

use crate::{scenarios, LifecycleStatusTotals, PhaseResultStatus};

use super::super::{escape_json, ScenarioExecutionResult};

pub(super) struct EvidenceSummaryCounts {
    pub(super) kolme_blocks_produced: u64,
    pub(super) proofs_anchored: u64,
    pub(super) proofs_verified: u64,
}

pub(super) fn valid_chain_dump_json() -> &'static str {
    r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#
}

pub(super) fn scenario_manifest_json(
    scenario: &scenarios::ScenarioDefinition,
    result: &ScenarioExecutionResult,
) -> String {
    let relative_path = scenario_artifact_relative_path(scenario.id);
    let relative_path_string = relative_path.to_string_lossy();
    format!(
        "{{\"id\":\"{}\",\"name\":\"{}\",\"status\":\"{}\",\"duration_seconds\":1,\"evidence_files\":[\"{}\"],\"verifiable_outputs\":{}}}",
        escape_json(scenario.id),
        escape_json(scenario.name),
        result.status.as_str(),
        escape_json(relative_path_string.as_ref()),
        scenario.verifiable_outputs.len()
    )
}

pub(super) fn evidence_summary_counts(
    scenario_totals: &LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> EvidenceSummaryCounts {
    if evidence_status == PhaseResultStatus::Pass {
        return EvidenceSummaryCounts {
            kolme_blocks_produced: std::cmp::max(1, scenario_totals.pass),
            proofs_anchored: scenario_totals.pass,
            proofs_verified: scenario_totals.pass,
        };
    }
    EvidenceSummaryCounts {
        kolme_blocks_produced: 0,
        proofs_anchored: 0,
        proofs_verified: 0,
    }
}

pub(super) fn scenario_artifact_json(
    scenario: &scenarios::ScenarioDefinition,
    result: &ScenarioExecutionResult,
) -> String {
    let scenario_token = normalized_scenario_token(scenario.id);
    format!(
        "{{\"scenario_id\":\"{}\",\"scenario_name\":\"{}\",\"status\":\"{}\",\"_verification\":{{\"evidence_hash\":\"sha256:{}-artifact\",\"captured_at\":\"2026-02-21T14:31:05Z\",\"source_node\":\"kamn-processor-1\",\"agent\":\"kamn-e2e-harness\",\"kolme_anchor\":{{\"tx_hash\":\"sha256:{}-tx\",\"block_height\":42,\"finality\":\"FINAL\"}}}}}}",
        escape_json(scenario.id),
        escape_json(scenario.name),
        result.status.as_str(),
        scenario_token,
        scenario_token
    )
}

fn scenario_artifact_relative_path(scenario_id: &str) -> PathBuf {
    let normalized = normalized_scenario_token(scenario_id);
    PathBuf::from(format!("scenario-{normalized}/artifact.json"))
}

fn normalized_scenario_token(scenario_id: &str) -> String {
    scenario_id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase()
}
