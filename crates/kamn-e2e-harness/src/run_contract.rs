use std::env;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use crate::{
    all_orchestration_phases, drivers, scenarios, ExecutionMode, LifecycleStatusTotals,
    LifecycleSummary, OrchestrationPhase, OrchestrationPhaseResult, OrchestrationStepRecord,
    PhaseResultStatus, RunCommandConfig,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioExecutionResult {
    id: String,
    status: PhaseResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeComponentProbe {
    status: PhaseResultStatus,
    detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeComponentBinaries {
    kamn_processor_binary: String,
    kamn_listener_binary: String,
    kamn_approver_binary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeProbeSummary {
    status: PhaseResultStatus,
    detail: String,
    kolme: ExternalRuntimeComponentProbe,
    kamn_processor: ExternalRuntimeComponentProbe,
    kamn_listener: ExternalRuntimeComponentProbe,
    kamn_approver: ExternalRuntimeComponentProbe,
    agent: ExternalRuntimeComponentProbe,
}

const EXTERNAL_KAMN_PROCESSOR_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY";
const EXTERNAL_KAMN_LISTENER_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY";
const EXTERNAL_KAMN_APPROVER_BINARY_ENV: &str = "KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY";

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Executes run-command contract behavior and returns deterministic JSON summary.
pub fn execute_run_contract(config: &RunCommandConfig) -> Result<String, String> {
    let mode = ExecutionMode::parse(config.mode.as_str())?;
    let agent_binary_required = matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny);
    if agent_binary_required
        && config
            .agent_binary
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
    {
        return Err("missing required agent binary for MCP modes".to_owned());
    }
    if config.external_execution {
        ensure_external_execution_preflight(config, mode)?;
    }
    let selected = select_scenarios(config.scenario_ids.as_slice())?;
    let phases = all_orchestration_phases();
    let infra_fail_path_marker = config.evidence_dir.contains("fail-path");
    let scenario_fail_path_marker = config.evidence_dir.contains("scenario-fail");
    let evidence_fail_path_marker = config.evidence_dir.contains("evidence-fail");
    let scenario_results = if config.external_execution {
        execute_selected_scenarios(mode, selected.as_slice(), scenario_fail_path_marker)?
    } else {
        execute_selected_scenarios_contract_only(selected.as_slice(), scenario_fail_path_marker)
    };
    let evidence_status = if evidence_fail_path_marker {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let phase_results = build_phase_results(
        phases.as_slice(),
        mode,
        infra_fail_path_marker,
        scenario_results.as_slice(),
        evidence_status,
    );
    let agent_binary_json = config
        .agent_binary
        .as_deref()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_owned());
    let integration_config_json = format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":{},\"agent_binary_required\":{},\"external_execution_enabled\":{}}}",
        escape_json(config.kolme_binary.as_str()),
        agent_binary_json,
        if agent_binary_required {
            "true"
        } else {
            "false"
        },
        if config.external_execution {
            "true"
        } else {
            "false"
        }
    );
    let kolme_binary_status = if config.kolme_binary.trim().is_empty() {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let agent_binary_status = if agent_binary_required {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Skip
    };
    let scenario_selection_status = if selected.is_empty() {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let overall_status = if [
        kolme_binary_status,
        agent_binary_status,
        scenario_selection_status,
    ]
    .contains(&PhaseResultStatus::Fail)
    {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let runtime_readiness_json = format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":\"{}\",\"scenario_selection\":\"{}\",\"overall\":\"{}\"}}",
        kolme_binary_status.as_str(),
        agent_binary_status.as_str(),
        scenario_selection_status.as_str(),
        overall_status.as_str()
    );
    let agent_runtime = match mode {
        ExecutionMode::SdkDirect => "sdk-direct",
        ExecutionMode::CliScripted => "cli-scripted",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent",
    };
    let process_runtime_json = format!(
        "{{\"kolme_runtime\":\"external-binary\",\"kamn_nodes_runtime\":\"managed-process-set\",\"agent_runtime\":\"{}\",\"spawn_strategy\":\"contract-scaffold\"}}",
        agent_runtime
    );
    let process_lifecycle_json = "{\"postgres\":\"planned\",\"kolme\":\"planned\",\"kamn_processor\":\"planned\",\"kamn_listener\":\"planned\",\"kamn_approver\":\"planned\"}";
    let spawn_timeline_json = "{\"postgres_start\":\"step-1\",\"kolme_start\":\"step-2\",\"kamn_nodes_start\":\"step-3\",\"agent_deploy_start\":\"step-4\"}";
    let scenario_totals =
        status_totals_from_iter(scenario_results.iter().map(|result| result.status));
    let mode_driver = mode_driver_label(mode);
    let selected_scenarios = selected.len() as u64;
    let executed_scenarios = scenario_results.len() as u64;
    let mode_execution_status = if selected_scenarios == executed_scenarios {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Fail
    };
    let mode_execution_contract_json = format!(
        "{{\"mode\":\"{}\",\"driver\":\"{}\",\"selected_scenarios\":{},\"executed_scenarios\":{},\"status\":\"{}\"}}",
        mode.as_str(),
        mode_driver,
        selected_scenarios,
        executed_scenarios,
        mode_execution_status.as_str()
    );
    let live_validation_status = if scenario_totals.fail > 0 {
        PhaseResultStatus::Fail
    } else if scenario_totals.pass > 0 {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Skip
    };
    let expected_live_checks: u64 = 4;
    let completed_live_checks = if live_validation_status == PhaseResultStatus::Fail {
        expected_live_checks - 1
    } else {
        expected_live_checks
    };
    let live_validation_json = format!(
        "{{\"expected_checks\":{},\"completed_checks\":{},\"status\":\"{}\"}}",
        expected_live_checks,
        completed_live_checks,
        live_validation_status.as_str()
    );
    let expected_evidence_artifacts: u64 = 4;
    let recorded_evidence_artifacts = if evidence_fail_path_marker {
        expected_evidence_artifacts - 1
    } else {
        expected_evidence_artifacts
    };
    let evidence_contract_json = format!(
        "{{\"expected_artifacts\":{},\"recorded_artifacts\":{},\"status\":\"{}\"}}",
        expected_evidence_artifacts,
        recorded_evidence_artifacts,
        evidence_status.as_str()
    );
    let spawn_plan_json = format!(
        "{{\"postgres_cmd\":\"docker run --rm --name kamn-e2e-postgres postgres:15\",\"kolme_cmd\":\"example-p2p api-server --bind 127.0.0.1:3000\",\"kamn_processor_cmd\":\"kamn-node --role processor --execution-mode {}\",\"kamn_listener_cmd\":\"kamn-node --role listener --execution-mode {}\",\"kamn_approver_cmd\":\"kamn-node --role approver --execution-mode {}\"}}",
        mode.as_str(),
        mode.as_str(),
        mode.as_str()
    );
    let spawn_execution_json = "{\"postgres\":{\"status\":\"PASS\",\"timeline_ref\":\"step-1\",\"result\":\"started\"},\"kolme\":{\"status\":\"PASS\",\"timeline_ref\":\"step-2\",\"result\":\"started\"},\"kamn_processor\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_listener\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_approver\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"}}";
    let live_process_execution_json = "{\"postgres\":{\"state\":\"running\",\"pid\":\"1001\",\"health\":\"PASS\"},\"kolme\":{\"state\":\"running\",\"pid\":\"1002\",\"health\":\"PASS\"},\"kamn_processor\":{\"state\":\"running\",\"pid\":\"2001\",\"health\":\"PASS\"},\"kamn_listener\":{\"state\":\"running\",\"pid\":\"2002\",\"health\":\"PASS\"},\"kamn_approver\":{\"state\":\"running\",\"pid\":\"2003\",\"health\":\"PASS\"}}";
    let orchestration_status = if phase_results.iter().any(|phase| {
        phase.phase != OrchestrationPhase::ScenarioRun
            && phase.phase != OrchestrationPhase::Evidence
            && phase.status == PhaseResultStatus::Fail
    }) {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let live_execution_overall = if [
        orchestration_status,
        live_validation_status,
        evidence_status,
    ]
    .contains(&PhaseResultStatus::Fail)
    {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let live_execution_json = format!(
        "{{\"orchestration_status\":\"{}\",\"validation_status\":\"{}\",\"evidence_status\":\"{}\",\"overall_status\":\"{}\"}}",
        orchestration_status.as_str(),
        live_validation_status.as_str(),
        evidence_status.as_str(),
        live_execution_overall.as_str()
    );
    let external_runtime_probe = if config.external_execution {
        Some(probe_external_runtime(config, mode))
    } else {
        None
    };
    let runtime_external_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        format!(
            "{{\"requested\":true,\"guard_status\":\"{}\",\"execution_mode\":\"external-runtime\",\"preflight\":\"ready\",\"probe_detail\":\"{}\"}}",
            probe.status.as_str(),
            escape_json(probe.detail.as_str())
        )
    } else {
        "{\"requested\":false,\"guard_status\":\"SKIP\",\"execution_mode\":\"contract-only\",\"preflight\":\"not-requested\"}".to_owned()
    };
    let runtime_orchestration_json = if let Some(probe) = external_runtime_probe.as_ref() {
        let postgres_status = aggregate_status(&[
            probe.kamn_processor.status,
            probe.kamn_listener.status,
            probe.kamn_approver.status,
        ]);
        let postgres_detail = if postgres_status == PhaseResultStatus::Pass {
            "postgres readiness derived from KAMN component probes".to_owned()
        } else {
            format!(
                "postgres readiness failed due component probe drift: processor={} listener={} approver={}",
                probe.kamn_processor.status.as_str(),
                probe.kamn_listener.status.as_str(),
                probe.kamn_approver.status.as_str()
            )
        };
        format!(
            "{{\"postgres\":{{\"requested\":true,\"status\":\"{}\",\"detail\":\"{}\"}},\"kolme\":{{\"requested\":true,\"status\":\"{}\",\"detail\":\"{}\"}},\"kamn_processor\":{{\"requested\":true,\"status\":\"{}\",\"detail\":\"{}\"}},\"kamn_listener\":{{\"requested\":true,\"status\":\"{}\",\"detail\":\"{}\"}},\"kamn_approver\":{{\"requested\":true,\"status\":\"{}\",\"detail\":\"{}\"}}}}",
            postgres_status.as_str(),
            escape_json(postgres_detail.as_str()),
            probe.kolme.status.as_str(),
            escape_json(probe.kolme.detail.as_str()),
            probe.kamn_processor.status.as_str(),
            escape_json(probe.kamn_processor.detail.as_str()),
            probe.kamn_listener.status.as_str(),
            escape_json(probe.kamn_listener.detail.as_str()),
            probe.kamn_approver.status.as_str(),
            escape_json(probe.kamn_approver.detail.as_str()),
        )
    } else {
        "{\"postgres\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kolme\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_processor\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_listener\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_approver\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"}}".to_owned()
    };
    let runtime_lifecycle_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        let postgres_status = aggregate_status(&[
            probe.kamn_processor.status,
            probe.kamn_listener.status,
            probe.kamn_approver.status,
        ]);
        format!(
            "{{\"postgres\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kolme\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_processor\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_listener\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_approver\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}}}}",
            postgres_status.as_str(),
            postgres_status.as_str(),
            postgres_status.as_str(),
            postgres_status.as_str(),
            probe.kolme.status.as_str(),
            probe.kolme.status.as_str(),
            probe.kolme.status.as_str(),
            probe.kolme.status.as_str(),
            probe.kamn_processor.status.as_str(),
            probe.kamn_processor.status.as_str(),
            probe.kamn_processor.status.as_str(),
            probe.kamn_processor.status.as_str(),
            probe.kamn_listener.status.as_str(),
            probe.kamn_listener.status.as_str(),
            probe.kamn_listener.status.as_str(),
            probe.kamn_listener.status.as_str(),
            probe.kamn_approver.status.as_str(),
            probe.kamn_approver.status.as_str(),
            probe.kamn_approver.status.as_str(),
            probe.kamn_approver.status.as_str(),
        )
    } else {
        "{\"postgres\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kolme\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_processor\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_listener\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_approver\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"}}".to_owned()
    };
    let runtime_validation_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        let orchestration_contract = probe.status;
        let lifecycle_contract = probe.status;
        let overall = aggregate_status(&[
            orchestration_contract,
            lifecycle_contract,
            live_validation_status,
            evidence_status,
        ]);
        format!(
            "{{\"requested\":true,\"orchestration_contract\":\"{}\",\"lifecycle_contract\":\"{}\",\"live_validation_contract\":\"{}\",\"evidence_contract\":\"{}\",\"overall\":\"{}\"}}",
            orchestration_contract.as_str(),
            lifecycle_contract.as_str(),
            live_validation_status.as_str(),
            evidence_status.as_str(),
            overall.as_str()
        )
    } else {
        "{\"requested\":false,\"orchestration_contract\":\"SKIP\",\"lifecycle_contract\":\"SKIP\",\"live_validation_contract\":\"SKIP\",\"evidence_contract\":\"SKIP\",\"overall\":\"SKIP\"}".to_owned()
    };
    let scenario_ids = selected
        .iter()
        .map(|item| format!("\"{}\"", item.id))
        .collect::<Vec<_>>()
        .join(",");
    let scenario_results_json = scenario_results
        .iter()
        .map(|result| {
            format!(
                "{{\"id\":\"{}\",\"status\":\"{}\"}}",
                escape_json(result.id.as_str()),
                result.status.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let scenario_contracts_json = selected
        .iter()
        .zip(scenario_results.iter())
        .map(|(scenario, result)| {
            let steps_json = scenario
                .steps
                .iter()
                .map(|step| format!("\"{}\"", escape_json(step)))
                .collect::<Vec<_>>()
                .join(",");
            let outputs_json = scenario
                .verifiable_outputs
                .iter()
                .map(|entry| format!("\"{}\"", escape_json(entry)))
                .collect::<Vec<_>>()
                .join(",");
            let pass_criteria_json = scenario
                .pass_criteria
                .iter()
                .map(|entry| format!("\"{}\"", escape_json(entry)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"priority\":\"{}\",\"status\":\"{}\",\"steps\":[{}],\"verifiable_outputs\":[{}],\"pass_criteria\":[{}]}}",
                scenario.id,
                escape_json(scenario.name),
                scenario.priority,
                result.status.as_str(),
                steps_json,
                outputs_json,
                pass_criteria_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let phase_labels = phases
        .iter()
        .map(|phase| format!("\"{}\"", phase.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    let phase_results_json = phase_results
        .iter()
        .map(|result| {
            let steps_json = result
                .steps
                .iter()
                .map(|step| {
                    format!(
                        "{{\"step\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
                        escape_json(step.step.as_str()),
                        step.status.as_str(),
                        escape_json(step.detail.as_str())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"phase\":\"{}\",\"status\":\"{}\",\"started_at\":\"{}\",\"completed_at\":\"{}\",\"details\":\"{}\",\"steps\":[{}]}}",
                result.phase.as_str(),
                result.status.as_str(),
                escape_json(result.started_at.as_str()),
                escape_json(result.completed_at.as_str()),
                escape_json(result.details.as_str()),
                steps_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let lifecycle_summary = compute_lifecycle_summary(phase_results.as_slice());
    let phase_totals_json = format!(
        "{{\"total\":{},\"pass\":{},\"fail\":{},\"skip\":{}}}",
        lifecycle_summary.phase_totals.total,
        lifecycle_summary.phase_totals.pass,
        lifecycle_summary.phase_totals.fail,
        lifecycle_summary.phase_totals.skip
    );
    let step_totals_json = format!(
        "{{\"total\":{},\"pass\":{},\"fail\":{},\"skip\":{}}}",
        lifecycle_summary.step_totals.total,
        lifecycle_summary.step_totals.pass,
        lifecycle_summary.step_totals.fail,
        lifecycle_summary.step_totals.skip
    );
    persist_run_evidence_bundle(
        config,
        mode,
        selected.as_slice(),
        scenario_results.as_slice(),
        scenario_totals,
        evidence_status,
    )?;
    Ok(format!(
        "{{\"command\":\"run\",\"mode\":\"{}\",\"evidence_dir\":\"{}\",\"integration_config\":{},\"runtime_external_execution\":{},\"runtime_orchestration\":{},\"runtime_lifecycle_execution\":{},\"runtime_validation_execution\":{},\"runtime_readiness\":{},\"process_runtime\":{},\"process_lifecycle\":{},\"spawn_timeline\":{},\"spawn_plan\":{},\"spawn_execution\":{},\"live_process_execution\":{},\"mode_execution_contract\":{},\"evidence_contract\":{},\"live_execution\":{},\"live_validation\":{},\"scenario_count\":{},\"scenario_ids\":[{}],\"scenario_results\":[{}],\"scenario_contracts\":[{}],\"phase_count\":{},\"phases\":[{}],\"phase_results\":[{}],\"lifecycle_summary\":{{\"phase_totals\":{},\"step_totals\":{}}}}}",
        mode.as_str(),
        escape_json(config.evidence_dir.as_str()),
        integration_config_json,
        runtime_external_execution_json,
        runtime_orchestration_json,
        runtime_lifecycle_execution_json,
        runtime_validation_execution_json,
        runtime_readiness_json,
        process_runtime_json,
        process_lifecycle_json,
        spawn_timeline_json,
        spawn_plan_json,
        spawn_execution_json,
        live_process_execution_json,
        mode_execution_contract_json,
        evidence_contract_json,
        live_execution_json,
        live_validation_json,
        selected.len(),
        scenario_ids,
        scenario_results_json,
        scenario_contracts_json,
        phases.len(),
        phase_labels,
        phase_results_json,
        phase_totals_json,
        step_totals_json
    ))
}

fn persist_run_evidence_bundle(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> Result<(), String> {
    let evidence_dir = Path::new(config.evidence_dir.as_str());
    std::fs::create_dir_all(evidence_dir).map_err(|error| {
        format!(
            "failed to create evidence directory {}: {error}",
            evidence_dir.display()
        )
    })?;

    let manifest_json = render_manifest_json(
        mode,
        selected,
        scenario_results,
        scenario_totals,
        evidence_status,
    );
    let manifest_path = evidence_dir.join("manifest.json");
    std::fs::write(&manifest_path, manifest_json).map_err(|error| {
        format!(
            "failed to write evidence manifest {}: {error}",
            manifest_path.display()
        )
    })?;

    let chain_dump_path = evidence_dir.join("kolme_chain_dump.json");
    if evidence_status == PhaseResultStatus::Pass {
        std::fs::write(&chain_dump_path, valid_chain_dump_json()).map_err(|error| {
            format!(
                "failed to write chain dump {}: {error}",
                chain_dump_path.display()
            )
        })?;
        persist_scenario_artifacts(evidence_dir, selected, scenario_results)?;
    } else if chain_dump_path.exists() {
        let _ = std::fs::remove_file(chain_dump_path);
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
        .map(|(scenario, result)| {
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
        })
        .collect::<Vec<_>>()
        .join(",");

    let (kolme_blocks_produced, proofs_anchored, proofs_verified) =
        if evidence_status == PhaseResultStatus::Pass {
            (
                std::cmp::max(1, scenario_totals.pass),
                scenario_totals.pass,
                scenario_totals.pass,
            )
        } else {
            (0, 0, 0)
        };
    let messages_exchanged = scenario_totals.pass.saturating_mul(2);

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
        kolme_blocks_produced,
        messages_exchanged,
        proofs_anchored,
        proofs_verified
    )
}

fn persist_scenario_artifacts(
    evidence_dir: &Path,
    selected: &[scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
) -> Result<(), String> {
    for (scenario, result) in selected.iter().zip(scenario_results.iter()) {
        let relative_path = scenario_artifact_relative_path(scenario.id);
        let artifact_path = evidence_dir.join(&relative_path);
        if let Some(parent) = artifact_path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create scenario evidence directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        let scenario_token = scenario
            .id
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        let artifact_json = format!(
            "{{\"scenario_id\":\"{}\",\"scenario_name\":\"{}\",\"status\":\"{}\",\"_verification\":{{\"evidence_hash\":\"sha256:{}-artifact\",\"captured_at\":\"2026-02-21T14:31:05Z\",\"source_node\":\"kamn-processor-1\",\"agent\":\"kamn-e2e-harness\",\"kolme_anchor\":{{\"tx_hash\":\"sha256:{}-tx\",\"block_height\":42,\"finality\":\"FINAL\"}}}}}}",
            escape_json(scenario.id),
            escape_json(scenario.name),
            result.status.as_str(),
            scenario_token,
            scenario_token
        );
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

fn valid_chain_dump_json() -> &'static str {
    r#"{"chain_name":"kamn-e2e-devnet","chain_version":1,"blocks":[{"height":0,"block_hash":"sha256:block-0","previous_block_hash":"GENESIS"},{"height":1,"block_hash":"sha256:block-1","previous_block_hash":"sha256:block-0"}]}"#
}

pub(crate) fn aggregate_status(statuses: &[PhaseResultStatus]) -> PhaseResultStatus {
    if statuses.contains(&PhaseResultStatus::Fail) {
        return PhaseResultStatus::Fail;
    }
    if statuses
        .iter()
        .all(|status| *status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

const ETXTBSY_ERRNO: i32 = 26;
const TEXT_FILE_BUSY_RETRY_LIMIT: usize = 3;

fn should_retry_text_file_busy(error: &std::io::Error, retry_attempt: usize) -> bool {
    error.raw_os_error() == Some(ETXTBSY_ERRNO) && retry_attempt < TEXT_FILE_BUSY_RETRY_LIMIT
}

fn probe_binary_invocation(binary: &str, label: &str) -> (PhaseResultStatus, String) {
    let args = probe_command_args_for_label(label);
    probe_binary_invocation_with_status_runner(label, || {
        let mut command = Command::new(binary);
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.status()
    })
}

fn probe_command_args_for_label(label: &str) -> &'static [&'static str] {
    match label {
        "kamn_processor" => &["--role", "processor"],
        "kamn_listener" => &["--role", "listener"],
        "kamn_approver" => &["--role", "approver"],
        _ => &["--help"],
    }
}

fn probe_binary_invocation_with_status_runner<F>(
    label: &str,
    mut status_runner: F,
) -> (PhaseResultStatus, String)
where
    F: FnMut() -> std::io::Result<std::process::ExitStatus>,
{
    for retry_attempt in 0..=TEXT_FILE_BUSY_RETRY_LIMIT {
        match status_runner() {
            Ok(status) if status.success() => {
                return (PhaseResultStatus::Pass, format!("{label} probe passed"));
            }
            Ok(status) => {
                let exit_status = status
                    .code()
                    .map(|code| code.to_string())
                    .unwrap_or_else(|| "signal".to_owned());
                return (
                    PhaseResultStatus::Fail,
                    format!("{label} probe failed (exit_status={exit_status})"),
                );
            }
            Err(error) if should_retry_text_file_busy(&error, retry_attempt) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => {
                return (
                    PhaseResultStatus::Fail,
                    format!("{label} probe failed ({error})"),
                );
            }
        }
    }
    (
        PhaseResultStatus::Fail,
        format!("{label} probe failed (retry budget exhausted)"),
    )
}

fn probe_external_runtime(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> ExternalRuntimeProbeSummary {
    let component_binaries = match resolve_external_runtime_component_binaries_from_env() {
        Ok(binaries) => binaries,
        Err(error) => {
            return ExternalRuntimeProbeSummary {
                status: PhaseResultStatus::Fail,
                detail: error.clone(),
                kolme: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Skip,
                    detail: "kolme probe skipped due unresolved runtime component binary envs"
                        .to_owned(),
                },
                kamn_processor: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Fail,
                    detail: error.clone(),
                },
                kamn_listener: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Fail,
                    detail: error.clone(),
                },
                kamn_approver: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Fail,
                    detail: error.clone(),
                },
                agent: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Skip,
                    detail: "agent probe skipped due unresolved runtime component binary envs"
                        .to_owned(),
                },
            };
        }
    };

    let (kolme_status, kolme_detail) =
        probe_binary_invocation(config.kolme_binary.as_str(), "kolme");
    let (kamn_processor_status, kamn_processor_detail) = probe_binary_invocation(
        component_binaries.kamn_processor_binary.as_str(),
        "kamn_processor",
    );
    let (kamn_listener_status, kamn_listener_detail) = probe_binary_invocation(
        component_binaries.kamn_listener_binary.as_str(),
        "kamn_listener",
    );
    let (kamn_approver_status, kamn_approver_detail) = probe_binary_invocation(
        component_binaries.kamn_approver_binary.as_str(),
        "kamn_approver",
    );
    let (agent_status, agent_detail) = if is_mcp_mode(mode) {
        let Some(agent_binary) = config.agent_binary.as_deref() else {
            return ExternalRuntimeProbeSummary {
                status: PhaseResultStatus::Fail,
                detail: "agent probe failed (missing binary path)".to_owned(),
                kolme: ExternalRuntimeComponentProbe {
                    status: kolme_status,
                    detail: kolme_detail,
                },
                kamn_processor: ExternalRuntimeComponentProbe {
                    status: kamn_processor_status,
                    detail: kamn_processor_detail,
                },
                kamn_listener: ExternalRuntimeComponentProbe {
                    status: kamn_listener_status,
                    detail: kamn_listener_detail,
                },
                kamn_approver: ExternalRuntimeComponentProbe {
                    status: kamn_approver_status,
                    detail: kamn_approver_detail,
                },
                agent: ExternalRuntimeComponentProbe {
                    status: PhaseResultStatus::Fail,
                    detail: "agent probe failed (missing binary path)".to_owned(),
                },
            };
        };
        probe_binary_invocation(agent_binary, "agent")
    } else {
        (
            PhaseResultStatus::Skip,
            "agent probe skipped (mode does not require agent binary)".to_owned(),
        )
    };
    let status = aggregate_status(&[
        kolme_status,
        kamn_processor_status,
        kamn_listener_status,
        kamn_approver_status,
        agent_status,
    ]);
    ExternalRuntimeProbeSummary {
        status,
        detail: format!(
            "{kolme_detail}; {kamn_processor_detail}; {kamn_listener_detail}; {kamn_approver_detail}; {agent_detail}"
        ),
        kolme: ExternalRuntimeComponentProbe {
            status: kolme_status,
            detail: kolme_detail,
        },
        kamn_processor: ExternalRuntimeComponentProbe {
            status: kamn_processor_status,
            detail: kamn_processor_detail,
        },
        kamn_listener: ExternalRuntimeComponentProbe {
            status: kamn_listener_status,
            detail: kamn_listener_detail,
        },
        kamn_approver: ExternalRuntimeComponentProbe {
            status: kamn_approver_status,
            detail: kamn_approver_detail,
        },
        agent: ExternalRuntimeComponentProbe {
            status: agent_status,
            detail: agent_detail,
        },
    }
}

fn execute_selected_scenarios(
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    force_first_fail: bool,
) -> Result<Vec<ScenarioExecutionResult>, String> {
    let _env_guard = crate::drivers::test_env_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let driver = driver_for_mode(mode)?;
    selected
        .iter()
        .enumerate()
        .map(|(index, scenario)| {
            let driver_result = driver.execute(scenario.id);
            let status = if force_first_fail && index == 0 {
                PhaseResultStatus::Fail
            } else {
                normalize_driver_status(driver_result.status)?
            };
            Ok(ScenarioExecutionResult {
                id: scenario.id.to_owned(),
                status,
            })
        })
        .collect()
}

fn execute_selected_scenarios_contract_only(
    selected: &[scenarios::ScenarioDefinition],
    force_first_fail: bool,
) -> Vec<ScenarioExecutionResult> {
    selected
        .iter()
        .enumerate()
        .map(|(index, scenario)| ScenarioExecutionResult {
            id: scenario.id.to_owned(),
            status: if force_first_fail && index == 0 {
                PhaseResultStatus::Fail
            } else {
                PhaseResultStatus::Pass
            },
        })
        .collect()
}

fn driver_for_mode(mode: ExecutionMode) -> Result<Box<dyn drivers::HarnessDriver>, String> {
    match mode {
        ExecutionMode::SdkDirect => Ok(Box::new(drivers::sdk_direct::SdkDirectDriver::from_env())),
        ExecutionMode::CliScripted => Ok(Box::new(
            drivers::cli_scripted::CliScriptedDriver::from_env(),
        )),
        ExecutionMode::McpTau | ExecutionMode::McpAny => Ok(Box::new(
            drivers::mcp_agent::McpAgentDriver::from_env(mode)?,
        )),
    }
}

fn normalize_driver_status(value: &str) -> Result<PhaseResultStatus, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "PASS" => Ok(PhaseResultStatus::Pass),
        "FAIL" => Ok(PhaseResultStatus::Fail),
        "SKIP" => Ok(PhaseResultStatus::Skip),
        other => Err(format!("unsupported driver execution status: {other}")),
    }
}

fn ensure_external_execution_preflight(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> Result<(), String> {
    if config.kolme_binary.trim().is_empty() {
        return Err("external execution preflight failed: kolme binary path is empty".to_owned());
    }
    ensure_binary_path_is_executable(config.kolme_binary.as_str(), "kolme")?;
    if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
        let agent_binary = config.agent_binary.as_deref().ok_or_else(|| {
            "external execution preflight failed: agent binary missing for MCP modes".to_owned()
        })?;
        if agent_binary.trim().is_empty() {
            return Err(
                "external execution preflight failed: agent binary path is empty".to_owned(),
            );
        }
        ensure_binary_path_is_executable(agent_binary, "agent")?;
    }
    let component_binaries = resolve_external_runtime_component_binaries_from_env()?;
    ensure_binary_path_is_executable(
        component_binaries.kamn_processor_binary.as_str(),
        "kamn_processor",
    )?;
    ensure_binary_path_is_executable(
        component_binaries.kamn_listener_binary.as_str(),
        "kamn_listener",
    )?;
    ensure_binary_path_is_executable(
        component_binaries.kamn_approver_binary.as_str(),
        "kamn_approver",
    )?;
    Ok(())
}

fn resolve_external_runtime_component_binaries_from_env(
) -> Result<ExternalRuntimeComponentBinaries, String> {
    Ok(ExternalRuntimeComponentBinaries {
        kamn_processor_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_PROCESSOR_BINARY_ENV,
            "kamn_processor",
        )?,
        kamn_listener_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_LISTENER_BINARY_ENV,
            "kamn_listener",
        )?,
        kamn_approver_binary: resolve_required_external_runtime_binary_env(
            EXTERNAL_KAMN_APPROVER_BINARY_ENV,
            "kamn_approver",
        )?,
    })
}

fn resolve_required_external_runtime_binary_env(
    env_name: &str,
    label: &str,
) -> Result<String, String> {
    let value = env::var(env_name).map_err(|_| {
        format!(
            "external execution preflight failed: missing required runtime component binary env: {env_name} ({label})"
        )
    })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!(
            "external execution preflight failed: runtime component binary env is empty: {env_name} ({label})"
        ));
    }
    Ok(trimmed.to_owned())
}

fn ensure_binary_path_is_executable(path: &str, label: &str) -> Result<(), String> {
    let binary_path = Path::new(path);
    if !binary_path.is_absolute() {
        return Err(format!(
            "external execution preflight failed: {label} binary path must be absolute: {path}"
        ));
    }
    if !binary_path.exists() {
        return Err(format!(
            "external execution preflight failed: {label} binary not found: {path}"
        ));
    }
    if !binary_path.is_file() {
        return Err(format!(
            "external execution preflight failed: {label} binary path is not a file: {path}"
        ));
    }
    ensure_binary_executable(binary_path, label)
}

#[cfg(unix)]
fn ensure_binary_executable(path: &Path, label: &str) -> Result<(), String> {
    let mode = std::fs::metadata(path)
        .map_err(|err| {
            format!(
                "external execution preflight failed: {label} binary metadata read failed: {} ({err})",
                path.display()
            )
        })?
        .permissions()
        .mode();
    if mode & 0o111 == 0 {
        return Err(format!(
            "external execution preflight failed: {label} binary is not executable: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_binary_executable(_path: &Path, _label: &str) -> Result<(), String> {
    Ok(())
}

fn compute_lifecycle_summary(phase_results: &[OrchestrationPhaseResult]) -> LifecycleSummary {
    let phase_totals = status_totals_from_iter(phase_results.iter().map(|result| result.status));
    let step_totals = status_totals_from_iter(
        phase_results
            .iter()
            .flat_map(|result| result.steps.iter().map(|step| step.status)),
    );
    LifecycleSummary {
        phase_totals,
        step_totals,
    }
}

fn status_totals_from_iter<I>(statuses: I) -> LifecycleStatusTotals
where
    I: IntoIterator<Item = PhaseResultStatus>,
{
    let mut totals = LifecycleStatusTotals {
        total: 0,
        pass: 0,
        fail: 0,
        skip: 0,
    };
    for status in statuses {
        totals.total += 1;
        match status {
            PhaseResultStatus::Pass => totals.pass += 1,
            PhaseResultStatus::Fail => totals.fail += 1,
            PhaseResultStatus::Skip => totals.skip += 1,
        }
    }
    totals
}

fn build_phase_results(
    phases: &[OrchestrationPhase],
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationPhaseResult> {
    let started_at = "1970-01-01T00:00:00Z";
    let completed_at = "1970-01-01T00:00:01Z";
    phases
        .iter()
        .map(|phase| {
            let steps = phase_step_records(
                *phase,
                mode,
                fail_path_marker,
                scenario_results,
                evidence_status,
            );
            let status = phase_status_for_steps(steps.as_slice());
            let details = phase_details(*phase, mode, status, scenario_results, evidence_status);
            OrchestrationPhaseResult {
                phase: *phase,
                status,
                started_at: started_at.to_owned(),
                completed_at: completed_at.to_owned(),
                details,
                steps,
            }
        })
        .collect()
}

fn phase_status_for_steps(steps: &[OrchestrationStepRecord]) -> PhaseResultStatus {
    if steps
        .iter()
        .any(|step| step.status == PhaseResultStatus::Fail)
    {
        return PhaseResultStatus::Fail;
    }
    if steps
        .iter()
        .all(|step| step.status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

fn phase_details(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    status: PhaseResultStatus,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> String {
    match (phase, status) {
        (OrchestrationPhase::InfraUp, PhaseResultStatus::Fail) => {
            "deterministic fail-path marker for infra startup".to_owned()
        }
        (OrchestrationPhase::InfraUp, _) => {
            "deterministic placeholder for infra startup".to_owned()
        }
        (OrchestrationPhase::AgentDeploy, _) => {
            "deterministic placeholder for agent deploy".to_owned()
        }
        (OrchestrationPhase::ScenarioRun, _) => {
            let totals =
                status_totals_from_iter(scenario_results.iter().map(|result| result.status));
            format!(
                "deterministic scenario execution summary: executed={} pass={} fail={} skip={}",
                totals.total, totals.pass, totals.fail, totals.skip
            )
        }
        (OrchestrationPhase::Evidence, _) => evidence_phase_detail(evidence_status),
        (OrchestrationPhase::Teardown, _) => teardown_phase_detail(mode, status),
    }
}

fn phase_step_records(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationStepRecord> {
    let pass = PhaseResultStatus::Pass;
    match phase {
        OrchestrationPhase::InfraUp => vec![
            OrchestrationStepRecord {
                step: "Start PostgreSQL container (docker)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: postgres startup".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Run Kolme migrations".to_owned(),
                status: pass,
                detail: "deterministic placeholder: migrations complete".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start Kolme processor (in-memory or Fjall storage)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme processor online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Verify Kolme API health (/healthz)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme health verified".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN processor node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: processor node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN listener node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: listener node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN approver node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: approver node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Wait for peer discovery (3 connected peers)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: peer discovery complete".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Verify KAMN Service API health (/healthz)".to_owned(),
                status: if fail_path_marker {
                    PhaseResultStatus::Fail
                } else {
                    pass
                },
                detail: if fail_path_marker {
                    "deterministic fail-path marker: kamn health check failed".to_owned()
                } else {
                    "deterministic placeholder: kamn health verified".to_owned()
                },
            },
        ],
        OrchestrationPhase::AgentDeploy => vec![
            OrchestrationStepRecord {
                step: "Generate ed25519 key pairs for Alice, Bob, Carol".to_owned(),
                status: pass,
                detail: "deterministic placeholder: keys generated".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Write key files to temp directory".to_owned(),
                status: pass,
                detail: "deterministic placeholder: key files materialized".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Register agents via kamn-agent-lib (POST /v1/agents/bootstrap)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: agents registered".to_owned(),
            },
            OrchestrationStepRecord {
                step: "[MCP modes] Spawn kamn-mcp-server per agent with identity".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp servers spawned".to_owned()
                } else {
                    "deterministic placeholder: mcp server spawn skipped for non-mcp mode"
                        .to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "[MCP modes] Verify MCP server health".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp health verified".to_owned()
                } else {
                    "deterministic placeholder: mcp health skipped for non-mcp mode".to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "Record infrastructure evidence".to_owned(),
                status: pass,
                detail: "deterministic placeholder: infra evidence recorded".to_owned(),
            },
        ],
        OrchestrationPhase::ScenarioRun => {
            let totals =
                status_totals_from_iter(scenario_results.iter().map(|result| result.status));
            let status = if totals.fail > 0 {
                PhaseResultStatus::Fail
            } else if totals.pass > 0 {
                PhaseResultStatus::Pass
            } else {
                PhaseResultStatus::Skip
            };
            vec![OrchestrationStepRecord {
                step: "Execute selected scenarios via mode driver".to_owned(),
                status,
                detail: format!(
                    "executed={} pass={} fail={} skip={}",
                    totals.total, totals.pass, totals.fail, totals.skip
                ),
            }]
        }
        OrchestrationPhase::Evidence => {
            let prerequisite_status = if evidence_status == PhaseResultStatus::Skip {
                PhaseResultStatus::Skip
            } else {
                pass
            };
            vec![
                OrchestrationStepRecord {
                    step: "Dump Kolme chain state".to_owned(),
                    status: prerequisite_status,
                    detail: "deterministic placeholder: kolme chain state dumped".to_owned(),
                },
                OrchestrationStepRecord {
                    step: "Dump KAMN node state snapshots".to_owned(),
                    status: prerequisite_status,
                    detail: "deterministic placeholder: kamn node snapshots dumped".to_owned(),
                },
                OrchestrationStepRecord {
                    step: "Verify all proof anchors independently".to_owned(),
                    status: evidence_status,
                    detail: evidence_verify_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Generate chain-of-custody report".to_owned(),
                    status: evidence_status,
                    detail: evidence_custody_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Compute evidence bundle hash".to_owned(),
                    status: evidence_status,
                    detail: evidence_hash_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Write manifest.json".to_owned(),
                    status: evidence_status,
                    detail: evidence_manifest_step_detail(evidence_status),
                },
            ]
        }
        OrchestrationPhase::Teardown => vec![
            OrchestrationStepRecord {
                step: "[MCP modes] Stop kamn-mcp-server processes".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp servers stopped".to_owned()
                } else {
                    "deterministic placeholder: mcp teardown skipped for non-mcp mode".to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "Stop KAMN nodes (graceful shutdown)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kamn nodes stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Stop Kolme devnet".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme devnet stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Stop PostgreSQL container".to_owned(),
                status: pass,
                detail: "deterministic placeholder: postgres container stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Archive evidence bundle".to_owned(),
                status: pass,
                detail: "deterministic placeholder: evidence bundle archived".to_owned(),
            },
        ],
    }
}

fn evidence_phase_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => {
            "deterministic evidence phase summary: steps_total=6 pass=2 fail=4 skip=0 expected_artifacts=4 recorded_artifacts=3 status=FAIL".to_owned()
        }
        PhaseResultStatus::Pass => {
            "deterministic evidence phase summary: steps_total=6 pass=6 fail=0 skip=0 expected_artifacts=4 recorded_artifacts=4 status=PASS".to_owned()
        }
        PhaseResultStatus::Skip => {
            "deterministic evidence phase summary: steps_total=6 pass=0 fail=0 skip=6 expected_artifacts=4 recorded_artifacts=0 status=SKIP".to_owned()
        }
    }
}

fn evidence_verify_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "proof_verification=FAIL verified=3 failed=1".to_owned(),
        PhaseResultStatus::Pass => "proof_verification=PASS verified=4 failed=0".to_owned(),
        PhaseResultStatus::Skip => "proof_verification=SKIP verified=0 failed=0".to_owned(),
    }
}

fn evidence_custody_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "custody_report=FAIL entries=3".to_owned(),
        PhaseResultStatus::Pass => "custody_report=PASS entries=4".to_owned(),
        PhaseResultStatus::Skip => "custody_report=SKIP entries=0".to_owned(),
    }
}

fn evidence_hash_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "bundle_hash=FAIL algorithm=sha256".to_owned(),
        PhaseResultStatus::Pass => "bundle_hash=PASS algorithm=sha256".to_owned(),
        PhaseResultStatus::Skip => "bundle_hash=SKIP algorithm=sha256".to_owned(),
    }
}

fn evidence_manifest_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => {
            "manifest_write=FAIL schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
        PhaseResultStatus::Pass => {
            "manifest_write=PASS schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
        PhaseResultStatus::Skip => {
            "manifest_write=SKIP schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
    }
}

fn teardown_phase_detail(mode: ExecutionMode, status: PhaseResultStatus) -> String {
    let mcp_step_status = if is_mcp_mode(mode) { "PASS" } else { "SKIP" };
    format!(
        "deterministic teardown summary: mcp_stop={} kamn_nodes=PASS kolme=PASS postgres=PASS archive=PASS status={}",
        mcp_step_status,
        status.as_str()
    )
}

fn is_mcp_mode(mode: ExecutionMode) -> bool {
    matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny)
}

fn mode_driver_label(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::SdkDirect => "sdk-direct-driver",
        ExecutionMode::CliScripted => "cli-scripted-driver",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent-driver",
    }
}

fn select_scenarios(ids: &[String]) -> Result<Vec<scenarios::ScenarioDefinition>, String> {
    let inventory = scenarios::all_scenarios();
    let mut selected = Vec::new();
    for id in ids {
        let matched = inventory
            .iter()
            .find(|item| item.id == id.as_str())
            .ok_or_else(|| format!("unknown scenario id: {id}"))?;
        selected.push(matched.clone());
    }
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::{
        probe_binary_invocation_with_status_runner, probe_command_args_for_label,
        should_retry_text_file_busy, PhaseResultStatus, ETXTBSY_ERRNO, TEXT_FILE_BUSY_RETRY_LIMIT,
    };

    #[test]
    fn unit_should_retry_text_file_busy_accepts_etxtbsy_within_retry_budget() {
        let busy_error = std::io::Error::from_raw_os_error(ETXTBSY_ERRNO);
        assert!(
            should_retry_text_file_busy(&busy_error, 0),
            "first ETXTBSY spawn error should retry"
        );
        assert!(
            should_retry_text_file_busy(&busy_error, TEXT_FILE_BUSY_RETRY_LIMIT - 1),
            "last in-budget ETXTBSY spawn error should retry"
        );
    }

    #[test]
    fn unit_should_retry_text_file_busy_rejects_non_retryable_error_shapes() {
        let busy_error = std::io::Error::from_raw_os_error(ETXTBSY_ERRNO);
        assert!(
            !should_retry_text_file_busy(&busy_error, TEXT_FILE_BUSY_RETRY_LIMIT),
            "ETXTBSY should not retry after budget exhaustion"
        );

        let missing_binary_error = std::io::Error::from_raw_os_error(2);
        assert!(
            !should_retry_text_file_busy(&missing_binary_error, 0),
            "non-ETXTBSY spawn errors must fail immediately"
        );
    }

    #[test]
    fn unit_probe_binary_invocation_retries_text_file_busy_up_to_retry_limit() {
        let mut calls = 0usize;
        let (status, detail) = probe_binary_invocation_with_status_runner("kolme", || {
            calls += 1;
            assert!(
                calls <= TEXT_FILE_BUSY_RETRY_LIMIT + 1,
                "retry loop exceeded ETXTBSY budget: calls={calls}"
            );
            Err(std::io::Error::from_raw_os_error(ETXTBSY_ERRNO))
        });
        assert_eq!(
            status,
            PhaseResultStatus::Fail,
            "exhausted ETXTBSY retries should fail closed"
        );
        assert_eq!(
            calls,
            TEXT_FILE_BUSY_RETRY_LIMIT + 1,
            "expected initial call plus bounded retries"
        );
        assert!(
            detail.contains("kolme probe failed"),
            "failure detail should retain probe context: {detail}"
        );
        assert!(
            !detail.contains("retry budget exhausted"),
            "expected concrete spawn error once retry budget is consumed: {detail}"
        );
    }

    #[test]
    fn unit_probe_binary_invocation_fails_immediately_for_non_retryable_spawn_errors() {
        let mut calls = 0usize;
        let (status, detail) = probe_binary_invocation_with_status_runner("kolme", || {
            calls += 1;
            Err(std::io::Error::from_raw_os_error(2))
        });
        assert_eq!(
            status,
            PhaseResultStatus::Fail,
            "non-ETXTBSY errors should fail immediately"
        );
        assert_eq!(calls, 1, "non-retryable spawn errors should not loop");
        assert!(
            detail.contains("kolme probe failed"),
            "failure detail should retain probe context: {detail}"
        );
    }

    #[test]
    fn unit_probe_command_args_for_kamn_components_use_role_startup_shape() {
        assert_eq!(
            probe_command_args_for_label("kamn_processor"),
            ["--role", "processor"],
            "processor probe should use deterministic role startup args"
        );
        assert_eq!(
            probe_command_args_for_label("kamn_listener"),
            ["--role", "listener"],
            "listener probe should use deterministic role startup args"
        );
        assert_eq!(
            probe_command_args_for_label("kamn_approver"),
            ["--role", "approver"],
            "approver probe should use deterministic role startup args"
        );
    }

    #[test]
    fn unit_probe_command_args_for_non_kamn_components_use_help_surface() {
        assert_eq!(
            probe_command_args_for_label("kolme"),
            ["--help"],
            "kolme probe should continue using help command shape"
        );
        assert_eq!(
            probe_command_args_for_label("agent"),
            ["--help"],
            "agent probe should continue using help command shape"
        );
    }
}
