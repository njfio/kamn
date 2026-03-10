use super::support::assert_doc_contains_all;

fn local_exclusion_and_budget_markers() -> &'static [&'static str] {
    &[
        "local-only fork sync/smoke run-mode commands remain excluded from ci-fast-gate.",
        "local Kolme API probe/smoke run-mode commands remain excluded from ci-fast-gate.",
        "local live API conformance harness run-mode commands remain excluded from ci-fast-gate.",
        "local fork bootstrap/readiness run-mode commands remain excluded from ci-fast-gate.",
        "local KAMN live runtime integration run-mode commands remain excluded from ci-fast-gate.",
        "local fork process lifecycle integration run-mode commands remain excluded from ci-fast-gate.",
        "local fork profile preflight run-mode commands remain excluded from ci-fast-gate.",
        "local fork self-test run-mode commands remain excluded from ci-fast-gate.",
        "local fork portability preflight run-mode commands remain excluded from ci-fast-gate.",
        "check_local_kolme_fork_portability_preflight_policy.py --report-file /tmp/kolme-local-fork-portability-preflight-summary.json",
        "local runtime-commit live run-mode commands remain excluded from ci-fast-gate.",
        "local native API parity live-proof run-mode commands remain excluded from ci-fast-gate.",
        "native parity fast/local command matrix remains synchronized across `README.md` and `docs/planning/kolme-devnet-ops.md`.",
        "baseline script inventory remains authoritative; any new script path must be documented by refreshing the committed baseline fixture in the same change.",
        "reason_codes=unexpected_current_scripts",
        "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json --phase contract --output-json /tmp/fast-gate-budget-delta-contract-report.json",
        "test_run_fast_gate_budget_delta_contract_lane.sh",
        "reason_codes=fast_gate_delta_threshold_file_stale",
        "reason_codes=fast_gate_delta_threshold_file_corrupt",
        "refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata",
        "check_non_kolme_wave_trend_test_loc_soft_budget.sh --waiver-file .ci/non_kolme_wave_trend_test_loc_soft_budget_waiver.json --output-json /tmp/non-kolme-wave-trend-test-loc-soft-budget-report.json",
        "reason_codes=delta_threshold_violation_unwaived",
        "reason_codes=delta_threshold_waiver_applied",
        "reason_codes=waiver_expired",
        "reason_codes=waiver_scope_mismatch",
        "native_libp2p_provider_marker=p2p-live-libp2p-provider:native",
        "libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only",
        "libp2p_fallback_markers_detected=none",
        "native_libp2p_provider_marker_contract_status=verified",
        "gate_policy_native_libp2p_provider_marker_mismatch",
        "gate_policy_libp2p_fallback_marker_blocklist_mismatch",
        "gate_policy_libp2p_fallback_markers_detected",
        "gate_policy_native_libp2p_provider_marker_contract_status_mismatch",
        "waiver_status=none|applied",
        "waived_reason_codes=none|...",
        "remediation=...",
    ]
}

fn extraction_threshold_markers() -> &'static [&'static str] {
    &[
        "test_check_kamn_node_main_rs_extraction_threshold.sh",
        "fixtures/ci/kamn_node_main_rs_extraction_thresholds.json",
        "check_kamn_node_main_rs_extraction_threshold.sh --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json",
        "check_kamn_node_main_rs_extraction_threshold.sh --exception-file .ci/kamn_node_main_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json",
        "policy_decision=GO|WARN|NO-GO",
        "exception_status=not-required|not-provided|applied|invalid|cap-exceeded",
        "reason_codes=main_rs_line_count_warn_threshold_exceeded",
        "reason_codes=main_rs_line_count_fail_threshold_exceeded",
        "reason_codes=main_rs_threshold_exception_applied",
        "reason_codes=main_rs_threshold_exception_expired",
        "reason_codes=main_rs_threshold_exception_cap_exceeded",
        "reason_codes=threshold_order_invalid",
        "test_check_kamn_sdk_service_rs_extraction_threshold.sh",
        "fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json",
        "check_kamn_sdk_service_rs_extraction_threshold.sh --output-json /tmp/kamn-sdk-service-rs-extraction-threshold-report.json",
        "check_kamn_sdk_service_rs_extraction_threshold.sh --exception-file .ci/kamn_sdk_service_rs_extraction_threshold_exception.json --output-json /tmp/kamn-sdk-service-rs-extraction-threshold-report.json",
        "reason_codes=service_rs_line_count_warn_threshold_exceeded",
        "reason_codes=service_rs_line_count_fail_threshold_exceeded",
        "reason_codes=service_rs_threshold_exception_applied",
        "reason_codes=service_rs_threshold_exception_expired",
        "reason_codes=service_rs_threshold_exception_cap_exceeded",
        "test_check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh",
        "fixtures/ci/kamn_node_runtime_orchestration_rs_extraction_thresholds.json",
        "check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json",
        "check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --exception-file .ci/kamn_node_runtime_orchestration_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json",
        "cargo test -p kamn-node --test main_module_extraction_contract",
        "reason_codes=runtime_orchestration_rs_line_count_warn_threshold_exceeded",
        "reason_codes=runtime_orchestration_rs_line_count_fail_threshold_exceeded",
        "reason_codes=runtime_orchestration_rs_threshold_exception_applied",
        "reason_codes=runtime_orchestration_rs_threshold_exception_expired",
        "reason_codes=runtime_orchestration_rs_threshold_exception_cap_exceeded",
        "test_check_touched_rust_size_policy.sh",
        "fixtures/ci/touched_rust_size_policy_thresholds.json",
        "fixtures/ci/touched_rust_size_policy_baseline.json",
        "check_touched_rust_size_policy.sh --output-json /tmp/touched-rust-size-policy-report.json",
        "reason_codes=touched_rust_size_policy_new_oversized_file",
        "reason_codes=touched_rust_size_policy_new_oversized_function",
        "reason_codes=touched_rust_size_policy_git_base_unavailable",
        "reason_codes=touched_rust_size_policy_threshold_invalid",
        "reason_codes=touched_rust_size_policy_baseline_invalid",
    ]
}

#[test]
fn doc_contains_local_exclusion_and_budget_markers() {
    assert_doc_contains_all(
        local_exclusion_and_budget_markers(),
        "local exclusion and budget",
    );
}

#[test]
fn doc_contains_extraction_threshold_markers() {
    assert_doc_contains_all(extraction_threshold_markers(), "extraction threshold");
}
