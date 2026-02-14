const DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn doc_contains_make_and_demo_scope_contract_rules() {
    assert!(DOC.contains("make check"));
    assert!(DOC.contains("make test"));
    assert!(DOC.contains("make demo"));
    assert!(DOC.contains("## Test Layering Policy Contract"));
    assert!(DOC.contains("scripts/ci/check_test_layering_policy.py"));
    assert!(DOC.contains("scripts/ci/test_check_test_layering_policy.sh"));
    assert!(DOC.contains("docs/planning/test_layering_policy.md"));
    assert!(DOC.contains("## Snapshot + Journal Durability Replay Contract"));
    assert!(DOC.contains("docs/planning/persistence_durability_model.md"));
    assert!(DOC.contains("cargo test -p kamn-core --lib journal"));
    assert!(DOC.contains("channel_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("message_lifecycle_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("task_operation_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("## Runtime Backpressure Enforcement Contract"));
    assert!(DOC.contains("docs/planning/runtime_backpressure_policy.md"));
    assert!(DOC.contains("cargo test -p kamn-core --lib backpressure"));
    assert!(DOC.contains("cargo test -p kamn-core --lib network_fault_simulation"));
    assert!(DOC.contains("runtime_backpressure_reject_new_enqueue"));
    assert!(DOC.contains("runtime_backpressure_purge_stale_peer_queue"));
    assert!(DOC.contains("## Lifecycle Property Shrinking Contract"));
    assert!(DOC.contains("docs/planning/property_invariant_matrix.md"));
    assert!(DOC.contains("cargo test -p kamn-core --test lifecycle_property_shrinking"));
    assert!(DOC.contains("cargo test -p kamn-core --test lifecycle_evidence_property_matrix"));
    assert!(DOC.contains("minimal failing prefix"));
    assert!(DOC.contains("## Coverage-Guided Parser Fuzz Contract"));
    assert!(DOC.contains("docs/planning/fuzz_harness_budget_policy.md"));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json"
    ));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json"
    ));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json"
    ));
    assert!(DOC.contains("run_input_mutation_coverage_guided_deep_lane.sh"));
    assert!(DOC.contains("runtime_input_mutation_coverage_guided_deep=skipped_local_only"));
    assert!(DOC.contains("KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY=true"));
    assert!(DOC.contains(
        "main_tests::functional_kolme_live_retry_emits_structured_retry_markers -- --exact"
    ));
    assert!(DOC.contains(
        "main_tests::functional_runtime_daemon_emits_structured_transition_markers -- --exact"
    ));
    assert!(DOC.contains("kolme.live.submit.retry"));
    assert!(DOC.contains("kolme.live.finality.retry"));
    assert!(DOC.contains("node.runtime.daemon.execute.start"));
    assert!(DOC.contains("node.runtime.daemon.execute.complete"));
    assert!(DOC.contains("layering_marker_missing"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane_tests"));
    assert!(DOC.contains("sdk-live-localhost-integration"));
    assert!(DOC.contains("KAMN_CI_TOOLS_FAST_MODE=true"));
    assert!(DOC.contains("test_check_legacy_ingress_parser_drift.sh"));
    assert!(DOC.contains(
        "check_legacy_ingress_parser_drift.sh --source-root crates/kamn-node/src --baseline-file fixtures/ci/legacy_ingress_parser_baseline.json --output-json /tmp/legacy-ingress-parser-drift-report.json"
    ));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_marker_count_increased"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_marker_new_file"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_baseline_missing"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_baseline_invalid"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(DOC.contains("scripts/ci/select_targets.sh"));
    assert!(DOC.contains("run_kolme_version_compatibility_contract_tests=true"));
    assert!(DOC.contains("test_run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("check_fast_gate_native_api_parity_policy.py"));
    assert!(DOC.contains("KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120"));
    assert!(DOC.contains("test_run_continuous_runtime_commit_contract_lane.sh"));
    assert!(DOC.contains("test_run_did_lifecycle_chain_adapter_contract_lane.sh"));
    assert!(DOC.contains("test_run_message_proof_anchoring_contract_lane.sh"));
    assert!(DOC.contains("test_run_managed_signer_startup_live_validation_contract_lane.sh"));
    assert!(DOC.contains("test_validate_continuous_runtime_commit_live.sh"));
    assert!(DOC.contains("test_validate_did_lifecycle_chain_adapter_live.sh"));
    assert!(DOC.contains("test_validate_message_proof_anchoring_live.sh"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave5_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave5_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave5_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave6_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave6_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave6_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave7_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave7_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave7_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave8_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave8_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave8_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave9_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave9_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave9_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave10_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_matrix.json"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_baseline.json"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_kolme_wave10_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_matrix.json"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_baseline.json"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_kolme_wave11_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave11_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave12_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave12_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave12_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave13_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave13_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave13_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave14_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave14_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave14_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave15_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave15_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave15_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave16_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave16_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave16_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave17_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave17_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave17_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave18_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave18_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave18_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("Non-Kolme bridge dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_bridge_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC.contains("Non-Kolme sdk dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_sdk_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC
        .contains("Non-Kolme lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_lightweight_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC.contains(
        "Non-Kolme wave-10 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave10_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-11 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave11_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-12 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave12_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-13 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave13_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-14 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave14_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-15 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave15_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-16 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave16_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-17 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave17_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-18 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave18_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains("run_local_fork_sync_metadata_lane.sh --mode run"));
    assert!(DOC.contains("run_local_fork_smoke_evidence_lane.sh --mode run"));
    assert!(DOC.contains(
        "run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2"
    ));
    assert!(DOC.contains("run_local_kolme_api_smoke_lane.sh --mode run"));
    assert!(DOC.contains("run_local_kolme_live_api_conformance_harness.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_live_api_conformance_contract_lane.sh --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json"
    ));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kamn_live_runtime_integration_contract_lane.sh --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json"
    ));
    assert!(DOC.contains(
        "run_local_kolme_fork_process_lifecycle_contract_lane.sh --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_profile_preflight_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_profile_preflight_contract_lane.sh"));
    assert!(DOC.contains("run_local_kolme_fork_self_test_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_self_test_contract_lane.sh"));
    assert!(DOC.contains("run_local_kolme_fork_portability_preflight_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_portability_preflight_contract_lane.sh"));
    assert!(DOC.contains("run_local_runtime_commit_live_lane.sh --mode run"));
    assert!(DOC.contains("run_local_native_api_parity_live_proof_lane.sh --mode run"));
    assert!(
        DOC.contains("--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast")
    );
    assert!(DOC.contains("test_run_local_runtime_commit_live_lane.sh"));
    assert!(DOC.contains("test_run_local_native_api_parity_live_proof_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_live_api_conformance_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_process_lifecycle_contract_lane.sh"));
    assert!(DOC.contains("run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("test_run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60"));
    assert!(DOC.contains("run_local_bootstrap_health_checks.sh"));
    assert!(DOC.contains("check_local_bootstrap_health_policy.py"));
    assert!(DOC.contains("run_local_bootstrap_health_checks_contract_lane.sh"));
    assert!(DOC.contains("test_check_local_bootstrap_health_policy.sh"));
    assert!(DOC.contains("test_run_local_bootstrap_health_checks_contract_lane.sh"));
    assert!(DOC.contains("run_local_e2e_integration_lane.sh"));
    assert!(DOC.contains("check_local_e2e_integration_policy.py"));
    assert!(DOC.contains("run_local_e2e_integration_contract_lane.sh"));
    assert!(DOC.contains("run_local_heavy_validation_matrix.sh"));
    assert!(DOC.contains("check_local_heavy_validation_matrix_policy.py"));
    assert!(DOC.contains("run_local_heavy_validation_matrix_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
    assert!(
        DOC.contains("local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.")
    );
    assert!(DOC.contains("kolme_local_heavy_lane_mode=local-only|manual-opt-in|not-applicable"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "local-only fork sync/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local Kolme API probe/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local live API conformance harness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork bootstrap/readiness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local KAMN live runtime integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork process lifecycle integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork profile preflight run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(
        DOC.contains("local fork self-test run-mode commands remain excluded from ci-fast-gate.")
    );
    assert!(DOC.contains(
        "local fork portability preflight run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "check_local_kolme_fork_portability_preflight_policy.py --report-file /tmp/kolme-local-fork-portability-preflight-summary.json"
    ));
    assert!(DOC.contains(
        "local runtime-commit live run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local native API parity live-proof run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "native parity fast/local command matrix remains synchronized across `README.md` and `docs/planning/kolme-devnet-ops.md`."
    ));
    assert!(DOC.contains(
        "baseline script inventory remains authoritative; any new script path must be documented by refreshing the committed baseline fixture in the same change."
    ));
    assert!(DOC.contains("reason_codes=unexpected_current_scripts"));
    assert!(DOC.contains(
        "run_fast_gate_budget_delta_contract_lane.sh --output-json /tmp/fast-gate-budget-delta-contract-report.json"
    ));
    assert!(DOC.contains("test_run_fast_gate_budget_delta_contract_lane.sh"));
    assert!(DOC.contains("reason_codes=fast_gate_delta_threshold_file_stale"));
    assert!(DOC.contains("reason_codes=fast_gate_delta_threshold_file_corrupt"));
    assert!(DOC.contains("refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata"));
    assert!(DOC.contains(
        "check_non_kolme_wave_trend_test_loc_soft_budget.sh --waiver-file .ci/non_kolme_wave_trend_test_loc_soft_budget_waiver.json --output-json /tmp/non-kolme-wave-trend-test-loc-soft-budget-report.json"
    ));
    assert!(DOC.contains("reason_codes=delta_threshold_violation_unwaived"));
    assert!(DOC.contains("reason_codes=delta_threshold_waiver_applied"));
    assert!(DOC.contains("reason_codes=waiver_expired"));
    assert!(DOC.contains("reason_codes=waiver_scope_mismatch"));
    assert!(DOC.contains("waiver_status=none|applied"));
    assert!(DOC.contains("waived_reason_codes=none|..."));
    assert!(DOC.contains("remediation=..."));
    assert!(DOC.contains("test_check_kamn_node_main_rs_extraction_threshold.sh"));
    assert!(DOC.contains("fixtures/ci/kamn_node_main_rs_extraction_thresholds.json"));
    assert!(DOC.contains(
        "check_kamn_node_main_rs_extraction_threshold.sh --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains(
        "check_kamn_node_main_rs_extraction_threshold.sh --exception-file .ci/kamn_node_main_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains("policy_decision=GO|WARN|NO-GO"));
    assert!(DOC.contains("exception_status=not-required|not-provided|applied|invalid|cap-exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_line_count_warn_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_line_count_fail_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_applied"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_expired"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_cap_exceeded"));
    assert!(DOC.contains("reason_codes=threshold_order_invalid"));
}

#[test]
fn regression_requires_make_and_selector_demo_contract_marker() {
    // Regression: #900
    assert!(DOC.contains("Regression: #900"));
    assert!(DOC.contains("make-target and selector workflow drift"));
    assert!(DOC.contains("Regression: #1419"));
    assert!(DOC.contains("Regression: #1431"));
    assert!(DOC.contains("Regression: #1682"));
    assert!(DOC.contains("Regression: #1687"));
    assert!(DOC.contains("Regression: #1697"));
    assert!(DOC.contains("Regression: #1702"));
    assert!(DOC.contains("Regression: #1707"));
    assert!(DOC.contains("Regression: #1692"));
    assert!(DOC.contains("Regression: #1441"));
    assert!(DOC.contains("Regression: #1451"));
    assert!(DOC.contains("Regression: #1467"));
    assert!(DOC.contains("Regression: #1468"));
    assert!(DOC.contains("Regression: #1482"));
    assert!(DOC.contains("Regression: #1483"));
    assert!(DOC.contains("Regression: #1488"));
    assert!(DOC.contains("Regression: #1489"));
    assert!(DOC.contains("Regression: #1494"));
    assert!(DOC.contains("Regression: #1462"));
    assert!(DOC.contains("Regression: #1466"));
    assert!(DOC.contains("Regression: #1497"));
    assert!(DOC.contains("Regression: #2694"));
    assert!(DOC.contains("Regression: #2690"));
    assert!(DOC.contains("Regression: #2691"));
    assert!(DOC.contains("Regression: #2692"));
    assert!(DOC.contains("Regression: #2693"));
    assert!(DOC.contains("Regression: #2093"));
    assert!(DOC.contains("Regression: #2095"));
    assert!(DOC.contains("Regression: #2658"));
    assert!(DOC.contains("Regression: #2703"));
    assert!(DOC.contains("Regression: #2705"));
    assert!(DOC.contains("Regression: #2711"));
    assert!(DOC.contains("Regression: #2714"));
    assert!(DOC.contains("Regression: #2717"));
    assert!(DOC.contains("Regression: #2720"));
    assert!(DOC.contains("Regression: #2723"));
    assert!(DOC.contains("Regression: #2726"));
    assert!(DOC.contains("Regression: #2729"));
    assert!(DOC.contains("Regression: #2732"));
    assert!(DOC.contains("Regression: #2735"));
    assert!(DOC.contains("Regression: #2738"));
    assert!(DOC.contains("Regression: #2741"));
}

#[test]
fn doc_contains_runtime_observability_endpoint_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Observability Endpoint Contract Lane"));
    assert!(DOC.contains("validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(DOC.contains("check_runtime_observability_endpoint_live_policy.sh"));
    assert!(DOC.contains("test_validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
}

#[test]
fn doc_contains_runtime_local_retry_diagnostics_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Local Retry/Diagnostics Contract Lane"));
    assert!(DOC.contains(
        "validate_local_retry_diagnostics_live.sh --mode dry-run --output-json /tmp/runtime-local-retry-diagnostics-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_RETRY_DIAGNOSTICS_OPT_IN=1 bash scripts/runtime/validate_local_retry_diagnostics_live.sh --mode run --output-json /tmp/runtime-local-retry-diagnostics-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_retry_diagnostics_live_policy.sh --report-file /tmp/runtime-local-retry-diagnostics-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/runtime-local-retry-diagnostics-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_retry_diagnostics_live_contract_lane.sh --output-json /tmp/runtime-local-retry-diagnostics-contract-lane-report.json --policy-output-json /tmp/runtime-local-retry-diagnostics-policy.json"
    ));
    assert!(DOC.contains("test_validate_local_retry_diagnostics_live.sh"));
    assert!(DOC.contains("test_check_local_retry_diagnostics_live_policy.sh"));
    assert!(DOC.contains("test_validate_local_retry_diagnostics_live_contract_lane.sh"));
    assert!(DOC.contains(
        "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC
        .contains("local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status"));
}

#[test]
fn doc_contains_runtime_local_signal_secret_hygiene_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Local Signal/Secret Hygiene Contract Lane"));
    assert!(DOC.contains(
        "validate_local_signal_secret_hygiene_live.sh --mode dry-run --output-json /tmp/runtime-local-signal-secret-hygiene-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_OPT_IN=1 bash scripts/runtime/validate_local_signal_secret_hygiene_live.sh --mode run --output-json /tmp/runtime-local-signal-secret-hygiene-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_signal_secret_hygiene_live_policy.sh --report-file /tmp/runtime-local-signal-secret-hygiene-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/runtime-local-signal-secret-hygiene-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_signal_secret_hygiene_live_contract_lane.sh --output-json /tmp/runtime-local-signal-secret-hygiene-contract-lane-report.json --policy-output-json /tmp/runtime-local-signal-secret-hygiene-policy.json"
    ));
    assert!(DOC.contains("test_validate_local_signal_secret_hygiene_live.sh"));
    assert!(DOC.contains("test_check_local_signal_secret_hygiene_live_policy.sh"));
    assert!(DOC.contains("test_validate_local_signal_secret_hygiene_live_contract_lane.sh"));
    assert!(DOC.contains(
        "local signal/secret hygiene run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
}

#[test]
fn doc_contains_runtime_service_api_axum_ingress_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Axum Ingress Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_axum_ingress_live.sh --output-json /tmp/service-api-axum-ingress-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_axum_ingress_live_policy.sh --report-file /tmp/service-api-axum-ingress-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-axum-ingress-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_axum_ingress_live_contract_lane.sh --output-json /tmp/service-api-axum-ingress-contract-lane-report.json --policy-output-json /tmp/service-api-axum-ingress-policy.json"
    ));
    assert!(DOC.contains("test_validate_service_api_axum_ingress_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_service_api_axum_ingress_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api axum ingress run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains("service_api_axum_policy_marker_missing:concurrency_status"));
}

#[test]
fn doc_contains_runtime_service_api_serde_payload_parity_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Serde Payload Parity Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_serde_payload_parity_live.sh --output-json /tmp/service-api-serde-payload-parity-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_serde_payload_parity_live_policy.sh --report-file /tmp/service-api-serde-payload-parity-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-serde-payload-parity-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_serde_payload_parity_live_contract_lane.sh --output-json /tmp/service-api-serde-payload-parity-contract-lane-report.json --policy-output-json /tmp/service-api-serde-payload-parity-policy.json"
    ));
    assert!(DOC.contains("test_validate_service_api_serde_payload_parity_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_service_api_serde_payload_parity_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api serde payload parity contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(
        DOC.contains("service_api_serde_payload_policy_marker_missing:route_payload_parity_status")
    );
}
