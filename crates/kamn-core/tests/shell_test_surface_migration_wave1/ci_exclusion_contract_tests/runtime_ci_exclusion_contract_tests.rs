use crate::ci_exclusion_contract_tests::support::{
    assert_ci_tools_and_doc, assert_workflow_and_fast_mode_exclusion, load_ci_exclusion_context,
};

#[test]
fn spec_c01_block_reconciliation_partition_rejoin_ci_exclusion_policy_markers() {
    assert_runtime_ci_exclusion(
        "bash scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh --mode run",
        "bash \"$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh\" --mode run",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_check_block_reconciliation_partition_rejoin_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh\"",
        ],
        "ci-tools block reconciliation command surface",
        "block reconciliation partition/rejoin run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "block reconciliation run-mode lane",
    );
}

#[test]
fn spec_c07_local_metrics_scrape_ci_exclusion_policy_markers() {
    assert_runtime_ci_exclusion(
        "bash scripts/runtime/validate_local_metrics_scrape_live.sh --mode run",
        "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_metrics_scrape_live_contract_lane.sh\"",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_local_metrics_scrape_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_metrics_scrape_live_contract_lane.sh\"",
        ],
        "ci-tools local metrics scrape command surface",
        "local metrics scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "local metrics scrape lane",
    );
}

#[test]
fn spec_c08_local_retry_diagnostics_ci_exclusion_policy_markers() {
    assert_runtime_ci_exclusion(
        "bash scripts/runtime/validate_local_retry_diagnostics_live.sh --mode run",
        "bash \"$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh\" --mode run",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh\"",
        ],
        "ci-tools local retry diagnostics command surface",
        "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "local retry diagnostics run-mode lane",
    );
}

fn assert_runtime_ci_exclusion(
    workflow_marker: &str,
    fast_mode_marker: &str,
    ci_tools_markers: &[&str],
    ci_tools_label: &str,
    doc_marker: &str,
    lane_label: &str,
) {
    let context = load_ci_exclusion_context();
    assert_workflow_and_fast_mode_exclusion(
        &context,
        workflow_marker,
        fast_mode_marker,
        lane_label,
    );
    assert_ci_tools_and_doc(
        &context,
        ci_tools_markers,
        ci_tools_label,
        doc_marker,
        lane_label,
    );
}
