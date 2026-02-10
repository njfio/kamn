const PLAN: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");

#[test]
fn plan_contains_triadic_smoke_contract_commands() {
    assert!(PLAN.contains("run_triadic_devnet_smoke.sh"));
    assert!(PLAN.contains("validate_triadic_devnet_smoke.py"));
    assert!(PLAN.contains("run_triadic_devnet_smoke_contract_lane.sh"));
}

#[test]
fn plan_contains_failover_sync_drill_lane_policy() {
    assert!(PLAN.contains("## Failover + Sync Drill Lane Policy"));
    assert!(PLAN.contains("select_failover_sync_drill_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_preflight_contract_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_deep_lane.sh"));
    assert!(PLAN.contains("run_failover_sync_drill_suite.sh"));
    assert!(PLAN.contains("kamn.runtime.failover-sync-drill-suite-report.v1"));
}

#[test]
fn plan_contains_runtime_commit_adapter_replay_lane_policy() {
    assert!(PLAN.contains("## Runtime Commit Adapter Replay/Finality Fast Lane"));
    assert!(PLAN.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(PLAN.contains("receipt_provider_mismatch"));
    assert!(PLAN.contains("receipt_not_final"));
}

#[test]
fn plan_contains_local_only_heavy_validation_matrix() {
    assert!(PLAN.contains("## Local-Only Heavy Kolme Validation Matrix"));
    assert!(PLAN.contains("run_local_heavy_validation_matrix.sh"));
    assert!(PLAN.contains("run_version_compatibility_replay_deep_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-heavy-validation-summary.v1"));
}

#[test]
fn regression_requires_failover_sync_budget_and_scheduled_cadence_guards() {
    // Regression: #788
    assert!(PLAN
        .contains("Failover/sync budget overruns and unscheduled deep-lane execution fail closed"));
}

#[test]
fn regression_requires_runtime_commit_adapter_reason_code_guard() {
    // Regression: #980
    assert!(PLAN.contains(
        "runtime commit adapter replay/finality reason-code drift fails closed (`Regression: #980`)."
    ));
}

#[test]
fn regression_requires_local_only_heavy_matrix_guard_marker() {
    // Regression: #1405
    assert!(PLAN.contains(
        "local-only heavy validation matrix requires explicit opt-in and remains excluded from PR fast-gate workflows (`Regression: #1405`)."
    ));
}
