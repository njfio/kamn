const RUNBOOK: &str = include_str!("../../../docs/foundation/mainnet-cutover-runbook.md");

#[test]
fn runbook_contains_manifest_schema_contract() {
    assert!(RUNBOOK.contains("## Manifest Schema Contract"));
    assert!(RUNBOOK.contains("kamn.mainnet-cutover.manifest.v1"));
    assert!(RUNBOOK.contains("fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json"));
    assert!(RUNBOOK.contains(
        "`id`, `order`, `role`, `status`, `approvals_required`, `approvals_received`, `approved_by`, `depends_on`, `rollback_ready`"
    ));
}

#[test]
fn runbook_contains_checkpoint_validator_contract() {
    assert!(RUNBOOK.contains("## Checkpoint Validator Contract"));
    assert!(RUNBOOK.contains("validate_mainnet_cutover_manifest.py"));
    assert!(RUNBOOK.contains("kamn.mainnet-cutover.validation-report.v1"));
    assert!(RUNBOOK.contains("validation_decision=GO"));
}

#[test]
fn runbook_contains_fast_contract_lane() {
    assert!(RUNBOOK.contains("## Fast Contract Lane"));
    assert!(RUNBOOK.contains("run_mainnet_cutover_contract_lane.sh"));
    assert!(RUNBOOK.contains("valid manifest acceptance path"));
    assert!(RUNBOOK.contains("insufficient approval evidence rejection"));
}

#[test]
fn runbook_contains_cutover_rollback_evidence_contract() {
    assert!(RUNBOOK.contains("## Cutover Rollback Evidence Contract"));
    assert!(RUNBOOK.contains("generate_cutover_rollback_evidence_bundle.sh"));
    assert!(RUNBOOK.contains("check_cutover_rollback_evidence_policy.sh"));
    assert!(RUNBOOK.contains("run_cutover_rollback_contract_lane.sh"));
    assert!(RUNBOOK.contains("run_cutover_rollback_deep_lane.sh"));
}

#[test]
fn runbook_contains_live_network_pilot_cutover_gates() {
    assert!(RUNBOOK.contains("## Live-Network Pilot Cutover Evidence Gates"));
    assert!(RUNBOOK.contains("run_live_network_smoke_lane.sh"));
    assert!(RUNBOOK.contains("run_live_network_pilot_deep_lane.sh"));
    assert!(RUNBOOK.contains("check_live_network_pilot_artifact_summary_policy.sh"));
}

#[test]
fn regression_requires_dependency_and_approval_rejection_policy() {
    // Regression: #705
    assert!(RUNBOOK.contains(
        "out-of-order or unresolved checkpoint dependencies force `NO-GO` (`Regression: #705`)."
    ));
    assert!(RUNBOOK.contains(
        "missing or insufficient checkpoint approvals force `NO-GO` (`Regression: #705`)."
    ));
}

#[test]
fn regression_requires_rollback_evidence_guards() {
    // Regression: #708
    assert!(RUNBOOK.contains(
        "missing failed-checkpoint rollback evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`)."
    ));
}

#[test]
fn regression_requires_live_network_pilot_cutover_guard_marker() {
    // Regression: #830
    assert!(RUNBOOK.contains(
        "pilot cutover progression is blocked when smoke/deep pilot evidence is missing or policy validation is `NO-GO` (`Regression: #830`)."
    ));
}
