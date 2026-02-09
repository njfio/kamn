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
fn regression_requires_dependency_and_approval_rejection_policy() {
    // Regression: #705
    assert!(RUNBOOK.contains(
        "out-of-order or unresolved checkpoint dependencies force `NO-GO` (`Regression: #705`)."
    ));
    assert!(RUNBOOK.contains(
        "missing or insufficient checkpoint approvals force `NO-GO` (`Regression: #705`)."
    ));
}
