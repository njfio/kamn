use crate::{LifecycleStatusTotals, PhaseResultStatus};

use super::super::super::super::{aggregate_status, escape_json, ExternalRuntimeProbeSummary};

pub(super) fn runtime_external_execution_json(
    external_execution: bool,
    probe: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    match (external_execution, probe) {
        (true, Some(probe)) => format!(
            "{{\"requested\":true,\"guard_status\":\"{}\",\"execution_mode\":\"external-runtime\",\"preflight\":\"ready\",\"probe_detail\":\"{}\"}}",
            probe.status.as_str(),
            escape_json(probe.detail.as_str())
        ),
        _ => "{\"requested\":false,\"guard_status\":\"SKIP\",\"execution_mode\":\"contract-only\",\"preflight\":\"not-requested\"}".to_owned(),
    }
}

pub(super) fn runtime_orchestration_json(
    external_execution: bool,
    probe: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    if !external_execution {
        return skipped_runtime_json();
    }
    match probe {
        Some(probe) => build_runtime_component_json(probe),
        None => missing_probe_json("orchestration_contract"),
    }
}

pub(super) fn runtime_lifecycle_execution_json(
    external_execution: bool,
    probe: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    if !external_execution {
        return skipped_lifecycle_json();
    }
    match probe {
        Some(probe) => build_runtime_lifecycle_json(probe),
        None => missing_probe_json("lifecycle_contract"),
    }
}

pub(super) fn runtime_validation_execution_json(
    external_execution: bool,
    probe: Option<&ExternalRuntimeProbeSummary>,
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    if !external_execution {
        return skipped_validation_json();
    }
    let Some(probe) = probe else {
        return missing_probe_validation_json(scenario_totals, evidence_status);
    };
    let validation = scenario_validation_status(scenario_totals);
    let overall = aggregate_status(&[probe.status, probe.status, validation, evidence_status]);
    format!(
        "{{\"requested\":true,\"orchestration_contract\":\"{}\",\"lifecycle_contract\":\"{}\",\"live_validation_contract\":\"{}\",\"evidence_contract\":\"{}\",\"overall\":\"{}\"}}",
        probe.status.as_str(),
        probe.status.as_str(),
        validation.as_str(),
        evidence_status.as_str(),
        overall.as_str()
    )
}

fn missing_probe_json(contract_field: &str) -> String {
    format!(
        "{{\"requested\":true,\"{contract_field}\":\"FAIL\",\"reason\":\"external runtime probe missing\"}}"
    )
}

fn missing_probe_validation_json(
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    let validation = scenario_validation_status(scenario_totals);
    let overall = aggregate_status(&[PhaseResultStatus::Fail, validation, evidence_status]);
    format!(
        "{{\"requested\":true,\"orchestration_contract\":\"FAIL\",\"lifecycle_contract\":\"FAIL\",\"live_validation_contract\":\"{}\",\"evidence_contract\":\"{}\",\"overall\":\"{}\",\"reason\":\"external runtime probe missing\"}}",
        validation.as_str(),
        evidence_status.as_str(),
        overall.as_str()
    )
}

fn build_runtime_component_json(probe: &ExternalRuntimeProbeSummary) -> String {
    let postgres_status = postgres_status(probe);
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
        postgres_status.as_str(), escape_json(postgres_detail.as_str()),
        probe.kolme.status.as_str(), escape_json(probe.kolme.detail.as_str()),
        probe.kamn_processor.status.as_str(), escape_json(probe.kamn_processor.detail.as_str()),
        probe.kamn_listener.status.as_str(), escape_json(probe.kamn_listener.detail.as_str()),
        probe.kamn_approver.status.as_str(), escape_json(probe.kamn_approver.detail.as_str())
    )
}

fn build_runtime_lifecycle_json(probe: &ExternalRuntimeProbeSummary) -> String {
    let postgres = postgres_status(probe);
    format!(
        "{{\"postgres\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kolme\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_processor\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_listener\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}},\"kamn_approver\":{{\"init\":\"{}\",\"spawn\":\"{}\",\"health_check\":\"{}\",\"ready\":\"{}\"}}}}",
        postgres.as_str(), postgres.as_str(), postgres.as_str(), postgres.as_str(),
        probe.kolme.status.as_str(), probe.kolme.status.as_str(), probe.kolme.status.as_str(), probe.kolme.status.as_str(),
        probe.kamn_processor.status.as_str(), probe.kamn_processor.status.as_str(), probe.kamn_processor.status.as_str(), probe.kamn_processor.status.as_str(),
        probe.kamn_listener.status.as_str(), probe.kamn_listener.status.as_str(), probe.kamn_listener.status.as_str(), probe.kamn_listener.status.as_str(),
        probe.kamn_approver.status.as_str(), probe.kamn_approver.status.as_str(), probe.kamn_approver.status.as_str(), probe.kamn_approver.status.as_str(),
    )
}

fn postgres_status(probe: &ExternalRuntimeProbeSummary) -> PhaseResultStatus {
    aggregate_status(&[
        probe.kamn_processor.status,
        probe.kamn_listener.status,
        probe.kamn_approver.status,
    ])
}

fn scenario_validation_status(scenario_totals: LifecycleStatusTotals) -> PhaseResultStatus {
    if scenario_totals.fail > 0 {
        return PhaseResultStatus::Fail;
    }
    if scenario_totals.pass > 0 {
        return PhaseResultStatus::Pass;
    }
    PhaseResultStatus::Skip
}

fn skipped_runtime_json() -> String {
    "{\"postgres\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kolme\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_processor\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_listener\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_approver\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"}}".to_owned()
}

fn skipped_lifecycle_json() -> String {
    "{\"postgres\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kolme\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_processor\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_listener\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_approver\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"}}".to_owned()
}

fn skipped_validation_json() -> String {
    "{\"requested\":false,\"orchestration_contract\":\"SKIP\",\"lifecycle_contract\":\"SKIP\",\"live_validation_contract\":\"SKIP\",\"evidence_contract\":\"SKIP\",\"overall\":\"SKIP\"}".to_owned()
}
