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
fn plan_contains_runtime_commit_block_fallback_fast_lane_policy() {
    assert!(PLAN.contains("## Runtime Commit Block Fallback Reconciliation Fast Lane"));
    assert!(PLAN.contains("run_block_fallback_reconciliation_contract_lane.sh"));
    assert!(PLAN.contains("kolme_runtime_commit_block_fallback"));
}

#[test]
fn plan_contains_local_fork_sync_metadata_lane() {
    assert!(PLAN.contains("## Deterministic Local Fork Sync Metadata Lane"));
    assert!(PLAN.contains("run_local_fork_sync_metadata_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-sync-metadata-summary.v1"));
    assert!(PLAN.contains("expected-remote-url https://github.com/njfio/kolme_fork.git"));
}

#[test]
fn plan_contains_local_fork_smoke_evidence_lane() {
    assert!(PLAN.contains("## Bounded Local Fork Smoke Evidence Lane"));
    assert!(PLAN.contains("run_local_fork_smoke_evidence_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-fork-smoke-evidence-summary.v1"));
    assert!(PLAN.contains("fork_smoke_command_timeout"));
}

#[test]
fn plan_contains_local_kolme_api_probe_lane() {
    assert!(PLAN.contains("## Deterministic Local Kolme API Probe Lane"));
    assert!(PLAN.contains("run_local_kolme_api_probe_lane.sh"));
    assert!(PLAN.contains("--fork-chain-version v0.15.2"));
    assert!(PLAN.contains("GET /fork-info?chain_version=<version>"));
    assert!(PLAN.contains("kamn.kolme.local-api-probe-summary.v1"));
}

#[test]
fn plan_contains_local_kolme_api_smoke_lane() {
    assert!(PLAN.contains("## Bounded Local-Only Kolme API Smoke Lane"));
    assert!(PLAN.contains("run_local_kolme_api_smoke_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-api-smoke-summary.v1"));
}

#[test]
fn plan_contains_local_runtime_commit_live_lane() {
    assert!(PLAN.contains("## Local Runtime Commit Live Proof Lane"));
    assert!(PLAN.contains("run_local_runtime_commit_live_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-runtime-commit-live-summary.v1"));
}

#[test]
fn plan_contains_local_native_api_parity_live_proof_lane() {
    assert!(PLAN.contains("## Local Native API Parity Live Proof Lane"));
    assert!(PLAN.contains("run_local_native_api_parity_live_proof_lane.sh"));
    assert!(
        PLAN.contains("--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast")
    );
    assert!(PLAN.contains("check_local_native_api_parity_live_proof_policy.py"));
    assert!(PLAN.contains("run_local_native_api_parity_live_proof_contract_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-native-api-parity-live-proof-summary.v1"));
}

#[test]
fn plan_contains_fast_gate_native_api_parity_lane() {
    assert!(PLAN.contains("## Fast-Gate Native API Parity Contract Lane"));
    assert!(PLAN.contains("run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(PLAN.contains("check_fast_gate_native_api_parity_policy.py"));
    assert!(PLAN.contains("kamn.kolme.fast-gate-native-api-parity-summary.v1"));
    assert!(PLAN.contains("KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS"));
    assert!(PLAN.contains("test_run_fast_gate_native_api_parity_contract_lane.sh"));
}

#[test]
fn plan_contains_local_only_heavy_validation_matrix() {
    assert!(PLAN.contains("## Local-Only Heavy Kolme Validation Matrix"));
    assert!(PLAN.contains("run_local_heavy_validation_matrix.sh"));
    assert!(PLAN.contains("run_local_bootstrap_health_checks.sh"));
    assert!(PLAN.contains("run_version_compatibility_replay_deep_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-heavy-validation-summary.v1"));
}

#[test]
fn plan_contains_deterministic_local_bootstrap_health_checks() {
    assert!(PLAN.contains("## Deterministic Local Bootstrap Health Checks"));
    assert!(PLAN.contains("run_local_bootstrap_health_checks.sh"));
    assert!(PLAN.contains("kamn.kolme.local-bootstrap-summary.v1"));
    assert!(PLAN.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
}

#[test]
fn plan_contains_local_only_heavy_e2e_lane() {
    assert!(PLAN.contains("## Local-Only Heavy End-to-End Lane"));
    assert!(PLAN.contains("run_local_e2e_integration_lane.sh"));
    assert!(PLAN.contains("kamn.kolme.local-e2e-integration-summary.v1"));
    assert!(PLAN.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(PLAN.contains("run_live_transport_parity_contract_lane.sh"));
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

#[test]
fn regression_requires_local_bootstrap_opt_in_guard_marker() {
    // Regression: #1417
    assert!(PLAN.contains(
        "deterministic bootstrap run mode fails closed without explicit local-only opt-in (`Regression: #1417`)."
    ));
}

#[test]
fn regression_requires_local_e2e_opt_in_guard_marker() {
    // Regression: #1418
    assert!(PLAN.contains(
        "local-only heavy E2E lane run mode fails closed without explicit local-only opt-in (`Regression: #1418`)."
    ));
}

#[test]
fn regression_requires_local_fork_sync_metadata_guard_marker() {
    // Regression: #1429
    assert!(PLAN.contains(
        "local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`)."
    ));
}

#[test]
fn regression_requires_local_fork_smoke_evidence_guard_marker() {
    // Regression: #1430
    assert!(PLAN.contains(
        "local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`)."
    ));
}

#[test]
fn regression_requires_local_kolme_api_probe_guard_marker() {
    // Regression: #1439
    assert!(PLAN.contains(
        "local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`)."
    ));
}

#[test]
fn regression_requires_local_kolme_api_smoke_guard_marker() {
    // Regression: #1440
    assert!(PLAN.contains(
        "local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`)."
    ));
}

#[test]
fn regression_requires_local_runtime_commit_live_guard_marker() {
    // Regression: #1450
    assert!(PLAN.contains(
        "local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`)."
    ));
}

#[test]
fn regression_requires_runtime_commit_block_fallback_guard_marker() {
    // Regression: #1464
    assert!(PLAN.contains(
        "block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`)."
    ));
}

#[test]
fn regression_requires_local_native_api_parity_live_proof_guard_marker() {
    // Regression: #1465
    assert!(PLAN.contains(
        "local native API parity live proof lane fails closed without local opt-in and on nonce/broadcast/finality timeout or command failures (`Regression: #1465`)."
    ));
}

#[test]
fn regression_requires_native_parity_docs_matrix_guard_marker() {
    // Regression: #1468
    assert!(PLAN.contains(
        "native parity fast/local command matrix docs drift remains fail-closed (`Regression: #1468`)."
    ));
}

#[test]
fn regression_requires_live_kolme_method_and_query_guard_marker() {
    // Regression: #1482
    assert!(PLAN.contains(
        "local probe fork-info query semantics and native parity broadcast method drift remain fail-closed (`Regression: #1482`)."
    ));
}
