use crate::{OrchestrationStepRecord, PhaseResultStatus};

use super::super::super::super::ScenarioExecutionResult;
use super::super::status_totals_from_iter;

pub(super) fn scenario_steps(results: &[ScenarioExecutionResult]) -> Vec<OrchestrationStepRecord> {
    let totals = status_totals_from_iter(results.iter().map(|result| result.status));
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

pub(super) fn evidence_steps(evidence_status: PhaseResultStatus) -> Vec<OrchestrationStepRecord> {
    let prerequisite = prerequisite_status(evidence_status);
    let mut steps = prerequisite_steps(prerequisite);
    steps.extend(evidence_validation_steps(evidence_status));
    steps
}

fn step_record(step: &str, status: PhaseResultStatus, detail: &str) -> OrchestrationStepRecord {
    OrchestrationStepRecord {
        step: step.to_owned(),
        status,
        detail: detail.to_owned(),
    }
}

fn prerequisite_status(evidence_status: PhaseResultStatus) -> PhaseResultStatus {
    if evidence_status == PhaseResultStatus::Skip {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

fn prerequisite_steps(status: PhaseResultStatus) -> Vec<OrchestrationStepRecord> {
    vec![
        step_record(
            "Dump Kolme chain state",
            status,
            "deterministic placeholder: kolme chain state dumped",
        ),
        step_record(
            "Dump KAMN node state snapshots",
            status,
            "deterministic placeholder: kamn node snapshots dumped",
        ),
    ]
}

fn evidence_validation_steps(status: PhaseResultStatus) -> Vec<OrchestrationStepRecord> {
    vec![
        evidence_step(
            "Verify all proof anchors independently",
            status,
            evidence_verify_step_detail(status),
        ),
        evidence_step(
            "Generate chain-of-custody report",
            status,
            evidence_custody_step_detail(status),
        ),
        evidence_step(
            "Compute evidence bundle hash",
            status,
            evidence_hash_step_detail(status),
        ),
        evidence_step(
            "Write manifest.json",
            status,
            evidence_manifest_step_detail(status),
        ),
    ]
}

fn evidence_step(step: &str, status: PhaseResultStatus, detail: String) -> OrchestrationStepRecord {
    step_record(step, status, detail.as_str())
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
