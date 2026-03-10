use super::super::DOC;
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_dependency_local_heavy_deep_scan_policy_checker_contract() {
    assert_contains_all(
        DOC,
        &[
            "## Dependency Local-Heavy Deep Scan Policy Checker Contract",
            "bash scripts/runtime/check_dependency_local_heavy_deep_scan_policy.sh --report-file /tmp/dependency-local-heavy-deep-scan-baseline.json --expected-final-decision GO --ci-fast-gate PASS --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --ci-tools-file scripts/ci/test_ci_tools.sh --workflow-file .github/workflows/ci-fast-gate.yml --output-json /tmp/dependency-local-heavy-deep-scan-policy-report.json",
            "bash scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh",
            "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version=kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1",
            "dependency_local_heavy_deep_scan_policy_reason_codes_csv=dependency_local_heavy_deep_scan_policy_required_field_missing,dependency_local_heavy_deep_scan_policy_marker_mismatch,dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch,dependency_local_heavy_deep_scan_policy_profile_contract_mismatch,dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch,dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift,dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift,ci_fast_gate_failed,dependency_local_heavy_deep_scan_policy_expected_decision_mismatch,dependency_local_heavy_deep_scan_policy_violation",
            "dependency_local_heavy_deep_scan_policy_strategy_doc_path=docs/ci/strategy.md",
            "dependency_local_heavy_deep_scan_policy_ops_doc_path=docs/ops/configuration.md",
            "dependency_local_heavy_deep_scan_policy_runner_report_schema_version=kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1",
            "dependency_local_heavy_deep_scan_policy_runner_reason_taxonomy_version=kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1",
            "dependency_local_heavy_deep_scan_policy_ci_dry_run_required_entry=bash \"$ROOT_DIR/scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh\"",
            "dependency_local_heavy_deep_scan_policy_ci_dry_run_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh\" --profile baseline --mode run",
            "dependency_local_heavy_deep_scan_policy_workflow_forbidden_entry=bash scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh --profile baseline --mode run",
        ],
        "dependency local heavy deep scan",
    );
}
