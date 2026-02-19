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
    assert!(DOC.contains(
        "bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh"
    ));
    assert!(DOC.contains(
        "`validate_unified_api_observability_local_heavy_live.sh --mode run --ci-fast-gate FAIL` must not appear in `.github/workflows/ci-fast-gate.yml` or `scripts/ci/test_ci_tools.sh` fast-mode block."
    ));
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
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json --phase contract --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json"
    ));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json --phase contract --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json"
    ));
    assert!(DOC.contains(
        "composite_gate_reason_taxonomy_version=kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "composite_gate_reason_codes_csv=dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded"
    ));
    assert!(DOC.contains("composite_gate_evidence_convergence_status=verified"));
    assert!(DOC.contains("composite_gate_ci_smoke_local_heavy_boundary_status=verified"));
    assert!(DOC.contains("composite_gate_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("composite_gate_local_heavy_execution_mode=not_requested"));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json"
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
        "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json --phase contract --output-json /tmp/fast-gate-budget-delta-contract-report.json"
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
    assert!(DOC.contains("native_libp2p_provider_marker=p2p-live-libp2p-provider:native"));
    assert!(DOC.contains(
        "libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only"
    ));
    assert!(DOC.contains("libp2p_fallback_markers_detected=none"));
    assert!(DOC.contains("native_libp2p_provider_marker_contract_status=verified"));
    assert!(DOC.contains("gate_policy_native_libp2p_provider_marker_mismatch"));
    assert!(DOC.contains("gate_policy_libp2p_fallback_marker_blocklist_mismatch"));
    assert!(DOC.contains("gate_policy_libp2p_fallback_markers_detected"));
    assert!(DOC.contains("gate_policy_native_libp2p_provider_marker_contract_status_mismatch"));
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
    assert!(DOC.contains("test_check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh"));
    assert!(
        DOC.contains("fixtures/ci/kamn_node_runtime_orchestration_rs_extraction_thresholds.json")
    );
    assert!(DOC.contains("check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json"));
    assert!(DOC.contains("check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --exception-file .ci/kamn_node_runtime_orchestration_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json"));
    assert!(DOC.contains("cargo test -p kamn-node --test main_module_extraction_contract"));
    assert!(
        DOC.contains("reason_codes=runtime_orchestration_rs_line_count_warn_threshold_exceeded")
    );
    assert!(
        DOC.contains("reason_codes=runtime_orchestration_rs_line_count_fail_threshold_exceeded")
    );
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_applied"));
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_expired"));
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_cap_exceeded"));
}

#[test]
fn doc_contains_node_runtime_startup_negative_matrix_fast_lane_contract_markers() {
    assert!(DOC.contains("## Node Runtime Startup Negative-Matrix Fast Lane"));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::cli_contract_tests::regression_3599_startup_signer_mode_negative_matrix_corpus -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node cli_tests::regression_3598_startup_paths_have_no_panic_control_flow -- --exact"
    ));
    assert!(DOC.contains("startup_negative_matrix_policy_marker_missing"));
    assert!(DOC.contains("must fail before network dispatch"));
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
    assert!(DOC.contains(
        "check_observability_endpoint_drift_contract.sh --output-json /tmp/observability-endpoint-drift-report.json"
    ));
    assert!(DOC.contains("test_validate_runtime_observability_endpoint_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_observability_endpoint_drift_contract.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains("ci-local contract-lane boundary rejects `--max-seconds > 240`."));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("endpoint_readiness_status=verified"));
    assert!(DOC.contains("stream_parity_status=verified"));
    assert!(DOC.contains("observability_source_marker_missing:legacy_tcp_listener_import"));
    assert!(DOC.contains("runtime_observability_endpoint_readiness_progress_stalled"));
    assert!(DOC.contains("runtime_observability_stream_parity_bypass_detected"));
    assert!(DOC.contains("ci_local_observability_endpoint_budget_boundary_exceeded"));
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
        "ci-local contract-lane budget remains fail-closed and rejects `--max-seconds > 240`."
    ));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("retry_readiness_status=verified"));
    assert!(DOC.contains("retry_backoff_status=verified"));
    assert!(DOC.contains("retry_jitter_parity_status=verified"));
    assert!(DOC.contains("retry_envelope_exhaustion_fail_closed_status=verified"));
    assert!(DOC.contains("reconnect_attempt_bound_status=verified"));
    assert!(DOC.contains("reconnect_backoff_bound_status=verified"));
    assert!(DOC.contains("retry_envelope_max_attempts=3"));
    assert!(DOC.contains("retry_envelope_max_backoff_seconds=8"));
    assert!(DOC.contains(
        "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC
        .contains("local_retry_diagnostics_policy_marker_missing:correlation_diagnostics_status"));
    assert!(DOC.contains("local_retry_readiness_progress_stalled"));
    assert!(DOC.contains("local_retry_backoff_jitter_parity_bypass_detected"));
    assert!(DOC.contains("local_retry_envelope_exhaustion_fail_closed_missing"));
    assert!(DOC.contains("local_retry_reconnect_attempt_bound_drift"));
    assert!(DOC.contains("local_retry_reconnect_backoff_bound_drift"));
    assert!(DOC.contains("ci_local_network_budget_boundary_exceeded"));
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
    assert!(DOC.contains("ci-local contract-lane boundary rejects `--max-seconds > 240`."));
    assert!(DOC.contains(
        "shutdown_reason_taxonomy_version=kamn.runtime.local-signal-shutdown-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "shutdown_reason_codes_csv=local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("signal_graceful_drain_status=verified"));
    assert!(DOC.contains(
        "local signal/secret hygiene run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("local_signal_shutdown_path_drift_detected"));
    assert!(DOC.contains("local_graceful_drain_bypass_detected"));
    assert!(DOC.contains("ci_local_signal_shutdown_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_local_metrics_scrape_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Local Metrics Scrape Contract Lane"));
    assert!(DOC.contains(
        "validate_local_metrics_scrape_live.sh --mode dry-run --output-json /tmp/local-metrics-scrape-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_METRICS_SCRAPE_OPT_IN=1 bash scripts/runtime/validate_local_metrics_scrape_live.sh --mode run --output-json /tmp/local-metrics-scrape-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_metrics_scrape_live_policy.sh --report-file /tmp/local-metrics-scrape-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-metrics-scrape-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_metrics_scrape_live_contract_lane.sh --output-json /tmp/local-metrics-scrape-live-contract-lane-report.json --policy-output-json /tmp/local-metrics-scrape-live-policy.json"
    ));
    assert!(DOC.contains("test_validate_local_metrics_scrape_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_local_metrics_scrape_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "local metrics scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains("local_metrics_scrape_policy_marker_missing:local_scrape_probe_status"));
    assert!(DOC.contains("local_metrics_scrape_policy_marker_missing:scrape_latency_budget_status"));
    assert!(DOC
        .contains("local_metrics_scrape_policy_metrics_emission_reason_taxonomy_version_mismatch"));
}

#[test]
fn doc_contains_runtime_libp2p_three_node_discovery_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Libp2p Three-Node Discovery Live Validation Contract Lane"));
    assert!(DOC.contains(
        "validate_libp2p_three_node_discovery_live.sh --mode dry-run --output-json /tmp/libp2p-three-node-discovery-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LIBP2P_THREE_NODE_DISCOVERY_LIVE_OPT_IN=1 bash scripts/runtime/validate_libp2p_three_node_discovery_live.sh --mode run --output-json /tmp/libp2p-three-node-discovery-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_libp2p_three_node_discovery_live_policy.sh --report-file /tmp/libp2p-three-node-discovery-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/libp2p-three-node-discovery-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_libp2p_three_node_discovery_live_contract_lane.sh --output-json /tmp/libp2p-three-node-discovery-live-contract-lane-report.json --policy-output-json /tmp/libp2p-three-node-discovery-live-policy.json"
    ));
    assert!(DOC.contains("test_validate_libp2p_three_node_discovery_live.sh"));
    assert!(DOC.contains("test_check_libp2p_three_node_discovery_live_policy.sh"));
    assert!(DOC.contains("test_validate_libp2p_three_node_discovery_live_contract_lane.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "libp2p three-node discovery run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains(
        "Kademlia bootstrap contracts are covered by `cargo test -p kamn-core --test p2p_kademlia_bootstrap`."
    ));
    assert!(DOC
        .contains("libp2p_three_node_discovery_policy_marker_missing:three_node_discovery_status"));
    assert!(DOC.contains("MissingKademliaBootstrapSeeds"));
}

#[test]
fn doc_contains_runtime_local_observability_scrape_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Local Observability Scrape Contract Lane"));
    assert!(DOC.contains(
        "validate_local_observability_scrape_live.sh --mode dry-run --output-json /tmp/local-observability-scrape-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN=1 bash scripts/runtime/validate_local_observability_scrape_live.sh --mode run --output-json /tmp/local-observability-scrape-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_observability_scrape_live_policy.sh --report-file /tmp/local-observability-scrape-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-observability-scrape-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_observability_scrape_live_contract_lane.sh --output-json /tmp/local-observability-scrape-live-contract-lane-report.json --policy-output-json /tmp/local-observability-scrape-live-policy.json"
    ));
    assert!(DOC.contains("test_validate_local_observability_scrape_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_local_observability_scrape_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains("docs/observability/streaming.md"));
    assert!(DOC.contains(
        "local observability scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains("local_observability_scrape_policy_marker_missing:scrape_probe_status"));
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
    assert!(DOC.contains(
        "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy.json --output-json /tmp/service-api-axum-ingress-convergence-report.json"
    ));
    assert!(DOC.contains("test_validate_service_api_axum_ingress_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_service_api_axum_ingress_live_policy.sh"));
    assert!(DOC.contains("test_check_service_api_axum_ingress_live_evidence_convergence.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api axum ingress run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains(
        "admission backpressure evidence convergence governance remains deterministic via:"
    ));
    assert!(DOC.contains(
        "admission saturation, in-flight, and queue-budget governance remains deterministic via:"
    ));
    assert!(DOC.contains(
        "admission decision taxonomy (accept/defer/reject) and runbook marker parity remains deterministic via:"
    ));
    assert!(DOC.contains("admission_inflight_budget_status=verified"));
    assert!(DOC.contains("admission_queue_budget_status=verified"));
    assert!(DOC.contains("admission_inflight_budget_limit=32"));
    assert!(DOC.contains("admission_queue_budget_limit=1"));
    assert!(DOC.contains(
        "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch"
    ));
    assert!(DOC.contains("admission_decision_taxonomy_status=verified"));
    assert!(DOC.contains("admission_decision_accept_status=verified"));
    assert!(DOC.contains("admission_decision_defer_status=verified"));
    assert!(DOC.contains("admission_decision_reject_status=verified"));
    assert!(DOC.contains(
        "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject"
    ));
    assert!(DOC.contains("admission_decision_taxonomy_mapping_status=verified"));
    assert!(DOC.contains("admission_decision_runbook_marker_parity_status=verified"));
    assert!(DOC.contains(
        "admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch"
    ));
    assert!(DOC.contains("service_api_axum_evidence_convergence_status=verified"));
    assert!(DOC.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "service_api_axum_evidence_reason_taxonomy_version=kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "service_api_axum_evidence_reason_codes_csv=service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch"
    ));
    assert!(DOC.contains("service_api_axum_policy_marker_missing:concurrency_status"));
    assert!(
        DOC.contains("service_api_axum_policy_admission_budget_reason_taxonomy_version_mismatch")
    );
    assert!(
        DOC.contains("service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch")
    );
    assert!(DOC.contains("service_api_axum_policy_admission_decision_reason_codes_csv_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_marker_missing:admission_decision_defer_status"));
    assert!(DOC.contains("service_api_axum_policy_admission_inflight_budget_limit_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_admission_queue_budget_limit_mismatch"));
    assert!(DOC.contains("service_api_axum_evidence_link_missing:source_report_file"));
    assert!(DOC.contains("service_api_axum_promotion_decision_reason_mapping_mismatch"));
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

#[test]
fn doc_contains_runtime_service_api_reason_code_compatibility_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Reason-Code Compatibility Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_reason_code_compatibility_live.sh --output-json /tmp/service-api-reason-code-compatibility-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_reason_code_compatibility_live_policy.sh --report-file /tmp/service-api-reason-code-compatibility-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-reason-code-compatibility-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_reason_code_compatibility_live_contract_lane.sh --output-json /tmp/service-api-reason-code-compatibility-contract-lane-report.json --policy-output-json /tmp/service-api-reason-code-compatibility-policy.json"
    ));
    assert!(
        DOC.contains("test_validate_service_api_reason_code_compatibility_live_contract_lane.sh")
    );
    assert!(DOC.contains("test_check_service_api_reason_code_compatibility_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api reason-code compatibility contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(
        DOC.contains("service_api_reason_code_policy_marker_missing:route_error_mapping_status")
    );
}

#[test]
fn doc_contains_runtime_service_api_validation_negative_matrix_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Validation Negative-Matrix Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_validation_negative_matrix_live.sh --mode dry-run --output-json /tmp/service-api-validation-negative-matrix-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_VALIDATION_NEGATIVE_MATRIX_OPT_IN=1 bash scripts/runtime/validate_service_api_validation_negative_matrix_live.sh --mode run --output-json /tmp/service-api-validation-negative-matrix-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_validation_negative_matrix_live_policy.sh --report-file /tmp/service-api-validation-negative-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-validation-negative-matrix-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_validation_negative_matrix_live_contract_lane.sh --output-json /tmp/service-api-validation-negative-matrix-contract-lane-report.json --policy-output-json /tmp/service-api-validation-negative-matrix-policy.json"
    ));
    assert!(
        DOC.contains("test_validate_service_api_validation_negative_matrix_live_contract_lane.sh")
    );
    assert!(DOC.contains("test_check_service_api_validation_negative_matrix_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api validation negative-matrix contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains(
        "service_api_validation_negative_matrix_policy_marker_missing:replay_guard_status"
    ));
}

#[test]
fn doc_contains_runtime_service_api_graceful_shutdown_drain_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Graceful-Shutdown Drain Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_graceful_shutdown_drain_live.sh --mode dry-run --output-json /tmp/service-api-graceful-shutdown-drain-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_GRACEFUL_SHUTDOWN_DRAIN_OPT_IN=1 bash scripts/runtime/validate_service_api_graceful_shutdown_drain_live.sh --mode run --output-json /tmp/service-api-graceful-shutdown-drain-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_graceful_shutdown_drain_live_policy.sh --report-file /tmp/service-api-graceful-shutdown-drain-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-graceful-shutdown-drain-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_graceful_shutdown_drain_live_contract_lane.sh --output-json /tmp/service-api-graceful-shutdown-drain-contract-lane-report.json --policy-output-json /tmp/service-api-graceful-shutdown-drain-policy.json"
    ));
    assert!(DOC.contains("test_validate_service_api_graceful_shutdown_drain_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_service_api_graceful_shutdown_drain_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api graceful-shutdown drain contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains(
        "service_api_graceful_shutdown_drain_policy_marker_missing:websocket_drain_status"
    ));
}

#[test]
fn doc_contains_runtime_service_api_shutdown_abrupt_close_regression_contract_lane_ci_mode_markers()
{
    assert!(DOC.contains("## Runtime Service API Shutdown Abrupt-Close Regression Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_shutdown_abrupt_close_regression_live.sh --mode dry-run --output-json /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_SHUTDOWN_ABRUPT_CLOSE_REGRESSION_OPT_IN=1 bash scripts/runtime/validate_service_api_shutdown_abrupt_close_regression_live.sh --mode run --output-json /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_shutdown_abrupt_close_regression_live_policy.sh --report-file /tmp/service-api-shutdown-abrupt-close-regression-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-shutdown-abrupt-close-regression-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh --output-json /tmp/service-api-shutdown-abrupt-close-regression-contract-lane-report.json --policy-output-json /tmp/service-api-shutdown-abrupt-close-regression-policy.json"
    ));
    assert!(DOC.contains(
        "test_validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh"
    ));
    assert!(DOC.contains("test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api shutdown abrupt-close regression contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC.contains(
        "service_api_shutdown_abrupt_close_regression_policy_marker_missing:abrupt_close_guard_status"
    ));
}

#[test]
fn doc_contains_runtime_service_api_prometheus_metrics_contract_lane_ci_mode_markers() {
    assert!(DOC.contains("## Runtime Service API Prometheus Metrics Contract Lane"));
    assert!(DOC.contains(
        "validate_service_api_prometheus_metrics_live.sh --mode dry-run --output-json /tmp/service-api-prometheus-metrics-live-summary.json"
    ));
    assert!(DOC.contains(
        "KAMN_LOCAL_PROMETHEUS_METRICS_OPT_IN=1 bash scripts/runtime/validate_service_api_prometheus_metrics_live.sh --mode run --output-json /tmp/service-api-prometheus-metrics-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_service_api_prometheus_metrics_live_policy.sh --report-file /tmp/service-api-prometheus-metrics-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-prometheus-metrics-policy.json"
    ));
    assert!(DOC.contains(
        "validate_service_api_prometheus_metrics_live_contract_lane.sh --output-json /tmp/service-api-prometheus-metrics-contract-lane-report.json --policy-output-json /tmp/service-api-prometheus-metrics-policy.json"
    ));
    assert!(DOC.contains("test_validate_service_api_prometheus_metrics_live_contract_lane.sh"));
    assert!(DOC.contains("test_check_service_api_prometheus_metrics_live_policy.sh"));
    assert!(DOC.contains("ci-fast-gate mode: fast"));
    assert!(DOC.contains("local-dev mode: local"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "service api prometheus metrics contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
    ));
    assert!(DOC
        .contains("service_api_prometheus_metrics_policy_marker_missing:metrics_contract_status"));
}

#[test]
fn doc_contains_ignored_test_and_script_budget_trend_composed_contract_markers() {
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json --phase contract --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json"
    ));
    assert!(DOC.contains("test_run_ignored_test_and_script_budget_trend_contract_lane.sh"));
    assert!(DOC.contains("ignored_test_metadata_stale_entry"));
    assert!(DOC.contains("combined_shell_surface_shell_line_total_delta_fail_exceeded"));
    assert!(DOC.contains("combined_shell_surface_ratio_fail_ceiling_exceeded"));
    assert!(DOC.contains("ignored_test_script_budget_trend_contract_status=pass|fail"));
}

#[test]
fn doc_contains_combined_shell_surface_baseline_refresh_workflow_markers() {
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_trigger_reason=combined_shell_surface_shell_line_total_delta_fail_exceeded"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_command=bash scripts/ci/generate_combined_shell_surface_trend_report.sh --budget-file .ci/script-surface-budget.env --script-baseline-file .ci/script-surface-baseline.env --combined-baseline-file fixtures/ci/combined_shell_surface_trend_baseline.json --output-json /tmp/combined-shell-surface-trend-report.json"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_contract=update fixtures/ci/combined_shell_surface_trend_baseline.json with report.current metrics in the same PR"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_validation=bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh"
    ));
}

#[test]
fn doc_contains_test_harness_structural_budget_reason_taxonomy_and_ci_smoke_markers() {
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_reason_taxonomy_version=kamn.ci.test-harness-loc-soft-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_reason_codes_csv=report_file_not_found,budget_file_not_found,baseline_file_not_found,trend_threshold_file_not_found,report_json_invalid,report_schema_mismatch,report_harness_script_count_invalid,report_harness_shell_line_total_invalid,budget_key_missing,budget_value_invalid,baseline_key_missing,baseline_value_invalid,trend_threshold_key_missing,trend_threshold_value_invalid,trend_threshold_order_invalid,harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded,harness_script_count_trend_warn_delta_exceeded,harness_shell_line_total_trend_warn_delta_exceeded,harness_script_count_trend_fail_delta_exceeded,harness_shell_line_total_trend_fail_delta_exceeded,trend_fail_enforcement_triggered"
    ));
    assert!(DOC.contains("test_harness_loc_soft_budget_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("test_harness_loc_soft_budget_reason_class=stable|budgeted|violation"));
    assert!(DOC.contains("test_harness_loc_soft_budget_ci_smoke_lane_cost_profile=low"));
    assert!(
        DOC.contains("test_harness_loc_soft_budget_ci_smoke_runtime_budget_status=within|exceeded")
    );
    assert!(DOC.contains("test_harness_loc_soft_budget_contract_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_contract_ci_smoke_runtime_budget_status=within|exceeded"
    ));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_contract_reason_key=test_harness_loc_soft_budget_contract_ok|test_harness_loc_soft_budget_contract_runtime_budget_exceeded"
    ));
}

#[test]
fn doc_contains_runtime_local_full_mode_live_validation_runtime_error_taxonomy_markers() {
    assert!(DOC.contains("## Runtime Local Full-Mode Live Validation Contract Lane"));
    assert!(DOC.contains(
        "validate_local_full_runtime_live.sh --mode dry-run --output-json /tmp/local-full-runtime-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_full_runtime_live_policy.sh --report-file /tmp/local-full-runtime-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_full_runtime_live_contract_lane.sh --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains("runtime_shutdown_gate_status=verified"));
    assert!(DOC.contains("runtime_fallback_classification_status=verified"));
    assert!(DOC.contains(
        "runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_status=verified"));
    assert!(DOC.contains("runtime_full_shutdown_gate_drift_detected"));
    assert!(DOC.contains("runtime_fallback_classification_unstable"));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_local_full_stack_runtime_budget_policy_markers() {
    assert!(DOC.contains("local_heavy_runtime_budget_status"));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_phase_module_parity_status=verified"));
    assert!(DOC.contains("runtime_extraction_evidence_output_status=verified"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_status=verified"));
    assert!(DOC.contains("elapsed_seconds"));
    assert!(DOC.contains("max_seconds"));
    assert!(DOC.contains("command_max_seconds"));
    assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_status_mismatch"));
    assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_exceeded"));
    assert!(DOC.contains("runtime_phase_module_parity_drift_detected"));
    assert!(DOC.contains("runtime_extraction_evidence_output_unstable"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_exceeded"));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_module_boundary_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status"
    ));
    assert!(DOC.contains("runtime_orchestration_dispatch_boundary_status=verified"));
    assert!(DOC.contains("runtime_daemon_phase_boundary_status=verified"));
    assert!(DOC.contains("runtime_kolme_live_boundary_status=verified"));
    assert!(DOC.contains("runtime_module_boundary_parity_status=verified"));
    assert!(DOC.contains("runtime_module_boundary_evidence_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_message_anchoring_ci_boundary_taxonomy_markers() {
    assert!(DOC.contains(
        "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required"
    ));
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
    assert!(DOC.contains("test_run_message_proof_anchoring_contract_lane.sh"));
    assert!(DOC.contains("test_validate_message_proof_anchoring_live.sh"));
}

#[test]
fn doc_contains_dependency_license_metadata_governance_taxonomy_and_boundary_markers() {
    assert!(DOC.contains(
        "metadata_governance_reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "metadata_governance_reason_codes_csv=expected_license_empty,no_crate_manifests_found,manifest_not_found,manifest_invalid_toml,package_section_missing,license_missing,license_mismatch,metadata_governance_local_heavy_opt_in_required"
    ));
    assert!(DOC.contains("metadata_governance_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "metadata_governance_reason_class=stable|metadata_mismatch|configuration|boundary|mixed"
    ));
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified|violation"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low|not-applicable"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=not_requested|opt_in|blocked"));
    assert!(DOC.contains(
        "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --lane-profile ci-smoke"
    ));
    assert!(DOC.contains(
        "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --lane-profile local-heavy --local-heavy-opt-in"
    ));
}

#[test]
fn doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "anti_flake_policy_reason_taxonomy_version=kamn.ci.anti-flake-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anti_flake_policy_reason_codes_csv=no_active_flaky_entries,active_flaky_entries_within_budget,active_flaky_entries_exceed_max,registry_validation_failed,registry_file_missing,expected_final_decision_mismatch,rerun_policy_fast_workflow_missing,rerun_policy_deep_workflow_missing,rerun_policy_bounded_retry_missing,rerun_policy_invariant_non_retry_missing,rerun_policy_excessive_retry_detected"
    ));
    assert!(DOC.contains("anti_flake_policy_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("anti_flake_policy_reason_class=stable|budgeted|violation"));
    assert!(DOC.contains(
        "check_anti_flake_policy.sh --registry-file .ci/flaky-tests.txt --expected-final-decision GO --max-active-entries 0 --fast-workflow-file .github/workflows/ci-fast-gate.yml --deep-workflow-file .github/workflows/ci-deep-validate.yml --output-json /tmp/anti-flake-policy-report.json"
    ));
}

#[test]
fn doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers() {
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified|violation"));
    assert!(DOC.contains("ci_smoke_performance_report_step_missing"));
    assert!(DOC.contains("ci_smoke_threshold_check_step_missing"));
    assert!(DOC.contains("local_heavy_opt_in_boundary_missing"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Incident go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("incident_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_codes_csv=incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("incident_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains("Regression: #4471"));
}

#[test]
fn doc_contains_live_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Live go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("live_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_live_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("live_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains(
        "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_csv=none|<csv>"));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "contracts.deployment_preflight_rotation_reason_taxonomy_version_required=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("contracts.go_no_go_gate_ci_local_boundary_contract_required=true"));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch"
    ));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch"
    ));
    assert!(DOC.contains("milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch"));
    assert!(DOC.contains("Regression: #4442"));
}

#[test]
fn doc_contains_transport_observability_tls_ci_smoke_convergence_governance() {
    assert!(DOC.contains("Transport/Observability/TLS CI smoke convergence governance"));
    assert!(DOC
        .contains("python3 scripts/ci/check_transport_observability_tls_ci_smoke_convergence.py"));
    assert!(DOC.contains("test_check_transport_observability_tls_ci_smoke_convergence.sh"));
    assert!(DOC.contains(
        "transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "transport_observability_tls_reason_codes_csv=transport_ci_smoke_composition_missing,observability_ci_smoke_composition_missing,tls_ci_smoke_composition_missing,transport_local_heavy_command_leaked_in_fast_mode,observability_local_heavy_command_leaked_in_fast_mode,tls_local_heavy_command_leaked_in_fast_mode,ci_fast_gate_transport_run_mode_not_excluded,ci_fast_gate_observability_run_mode_not_excluded,ci_fast_gate_tls_deep_lane_not_excluded,ci_strategy_convergence_markers_missing,production_plan_convergence_markers_missing,transport_observability_tls_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("transport_observability_tls_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("transport_observability_tls_local_heavy_max_seconds=900"));
    assert!(DOC.contains("transport_observability_tls_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("transport_observability_tls_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("transport_ci_smoke_composition_missing"));
    assert!(DOC.contains("observability_ci_smoke_composition_missing"));
    assert!(DOC.contains("tls_ci_smoke_composition_missing"));
    assert!(DOC.contains("transport_observability_tls_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("Regression: #4299"));
}

#[test]
fn doc_contains_admission_backpressure_ci_smoke_convergence_governance() {
    assert!(DOC.contains("### Admission-Backpressure CI smoke convergence governance"));
    assert!(DOC.contains("python3 scripts/ci/check_admission_backpressure_ci_smoke_convergence.py"));
    assert!(
        DOC.contains("bash scripts/ci/test_check_admission_backpressure_ci_smoke_convergence.sh")
    );
    assert!(DOC.contains(
        "admission_backpressure_ci_smoke_reason_taxonomy_version=kamn.ci.admission-backpressure-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_backpressure_ci_smoke_reason_codes_csv=service_api_axum_policy_ci_smoke_composition_missing,service_api_axum_contract_lane_ci_smoke_composition_missing,service_api_axum_run_command_leaked_in_fast_mode,ci_fast_gate_service_api_axum_run_command_not_excluded,ci_strategy_admission_backpressure_convergence_markers_missing,production_plan_admission_backpressure_convergence_markers_missing,admission_backpressure_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("admission_backpressure_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("admission_backpressure_local_heavy_max_seconds=900"));
    assert!(DOC.contains("admission_backpressure_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("admission_backpressure_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("service_api_axum_run_command_leaked_in_fast_mode"));
    assert!(DOC.contains("ci_fast_gate_service_api_axum_run_command_not_excluded"));
    assert!(DOC.contains("admission_backpressure_ci_smoke_seconds_exceeded"));
}

#[test]
fn doc_contains_sqlite_crash_replay_ci_smoke_convergence_governance() {
    assert!(DOC.contains("### SQLite Crash-Replay CI smoke convergence governance"));
    assert!(DOC.contains("python3 scripts/ci/check_sqlite_crash_recovery_ci_smoke_convergence.py"));
    assert!(
        DOC.contains("bash scripts/ci/test_check_sqlite_crash_recovery_ci_smoke_convergence.sh")
    );
    assert!(DOC.contains(
        "sqlite_crash_recovery_ci_smoke_reason_taxonomy_version=kamn.ci.sqlite-crash-recovery-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "sqlite_crash_recovery_ci_smoke_reason_codes_csv=sqlite_crash_recovery_validate_ci_smoke_composition_missing,sqlite_crash_recovery_policy_ci_smoke_composition_missing,sqlite_crash_recovery_contract_lane_ci_smoke_composition_missing,sqlite_crash_recovery_evidence_ci_smoke_composition_missing,sqlite_crash_recovery_run_mode_command_leaked_in_fast_mode,ci_fast_gate_sqlite_crash_recovery_run_mode_not_excluded,ci_strategy_sqlite_crash_recovery_convergence_markers_missing,production_plan_sqlite_crash_recovery_convergence_markers_missing,sqlite_crash_recovery_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("sqlite_crash_recovery_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("sqlite_crash_recovery_local_heavy_max_seconds=900"));
    assert!(DOC.contains("sqlite_crash_recovery_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("sqlite_crash_recovery_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("sqlite_crash_recovery_run_mode_command_leaked_in_fast_mode"));
    assert!(DOC.contains("ci_fast_gate_sqlite_crash_recovery_run_mode_not_excluded"));
    assert!(DOC.contains("sqlite_crash_recovery_ci_smoke_seconds_exceeded"));
}

#[test]
fn doc_contains_failover_drift_ci_smoke_convergence_governance() {
    assert!(DOC.contains("### Failover Drift CI smoke convergence governance"));
    assert!(DOC.contains("python3 scripts/ci/check_failover_drift_ci_smoke_convergence.py"));
    assert!(DOC.contains("bash scripts/ci/test_check_failover_drift_ci_smoke_convergence.sh"));
    assert!(DOC.contains(
        "failover_drift_ci_smoke_reason_taxonomy_version=kamn.ci.failover-drift-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "failover_drift_ci_smoke_reason_codes_csv=failover_selector_ci_smoke_composition_missing,failover_preflight_ci_smoke_composition_missing,failover_deep_lane_guard_ci_smoke_composition_missing,failover_suite_ci_smoke_composition_missing,failover_deep_lane_run_command_leaked_in_fast_mode,ci_fast_gate_failover_deep_lane_not_excluded,ci_strategy_failover_convergence_markers_missing,production_plan_failover_convergence_markers_missing,failover_drift_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("failover_drift_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("failover_drift_local_heavy_max_seconds=900"));
    assert!(DOC.contains("failover_drift_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("failover_drift_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("failover_deep_lane_run_command_leaked_in_fast_mode"));
    assert!(DOC.contains("ci_fast_gate_failover_deep_lane_not_excluded"));
    assert!(DOC.contains("failover_drift_ci_smoke_seconds_exceeded"));
}

#[test]
fn doc_contains_websocket_session_ci_smoke_convergence_governance() {
    assert!(DOC.contains("### Websocket Session CI smoke convergence governance"));
    assert!(DOC.contains("python3 scripts/ci/check_websocket_session_ci_smoke_convergence.py"));
    assert!(DOC.contains("bash scripts/ci/test_check_websocket_session_ci_smoke_convergence.sh"));
    assert!(DOC.contains(
        "websocket_session_ci_smoke_reason_taxonomy_version=kamn.ci.websocket-session-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "websocket_session_ci_smoke_reason_codes_csv=websocket_validate_ci_smoke_composition_missing,websocket_policy_ci_smoke_composition_missing,websocket_contract_ci_smoke_composition_missing,websocket_session_drill_run_command_leaked_in_fast_mode,ci_fast_gate_websocket_session_drill_not_excluded,ci_strategy_websocket_session_convergence_markers_missing,production_plan_websocket_session_convergence_markers_missing,websocket_session_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("websocket_session_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("websocket_session_local_heavy_max_seconds=900"));
    assert!(DOC.contains("websocket_session_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("websocket_session_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("websocket_session_drill_run_command_leaked_in_fast_mode"));
    assert!(DOC.contains("ci_fast_gate_websocket_session_drill_not_excluded"));
    assert!(DOC.contains("websocket_session_ci_smoke_seconds_exceeded"));
}

#[test]
fn doc_contains_partition_finality_ci_smoke_convergence_governance() {
    assert!(DOC.contains("### Partition-Finality CI smoke convergence governance"));
    assert!(DOC.contains("python3 scripts/ci/check_partition_finality_ci_smoke_convergence.py"));
    assert!(DOC.contains("bash scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh"));
    assert!(DOC.contains(
        "partition_finality_ci_smoke_reason_taxonomy_version=kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "partition_finality_ci_smoke_reason_codes_csv=libp2p_validate_ci_smoke_composition_missing,libp2p_policy_ci_smoke_composition_missing,libp2p_contract_lane_ci_smoke_composition_missing,libp2p_evidence_ci_smoke_composition_missing,partition_finality_run_mode_command_leaked_in_fast_mode,ci_fast_gate_partition_finality_run_mode_not_excluded,ci_strategy_partition_finality_convergence_markers_missing,production_plan_partition_finality_convergence_markers_missing,partition_finality_ci_smoke_seconds_exceeded"
    ));
    assert!(DOC.contains("partition_finality_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("partition_finality_local_heavy_max_seconds=900"));
    assert!(DOC.contains("partition_finality_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("partition_finality_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("libp2p_evidence_ci_smoke_composition_missing"));
    assert!(DOC.contains("partition_finality_run_mode_command_leaked_in_fast_mode"));
    assert!(DOC.contains("ci_fast_gate_partition_finality_run_mode_not_excluded"));
    assert!(DOC.contains("partition_finality_ci_smoke_seconds_exceeded"));
}

#[test]
fn doc_contains_persistence_adapter_integrity_ci_boundary_markers() {
    assert!(DOC.contains("## Persistence Adapter Integrity + CI Boundary Fast Lane"));
    assert!(DOC.contains("test_validate_persistence_adapters_live.sh"));
    assert!(DOC.contains(
        "persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation"
    ));
    assert!(DOC.contains("persistence_ci_smoke_local_heavy_boundary_status=verified"));
    assert!(DOC.contains("persistence_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("persistence_local_heavy_execution_mode=opt_in"));
}

#[test]
fn doc_contains_cutover_ci_exclusion_policy_contract_markers() {
    assert!(DOC.contains("## Cutover Rollback CI Exclusion Policy Contract"));
    assert!(DOC.contains("python3 scripts/cutover/check_cutover_ci_exclusion_policy.py"));
    assert!(DOC.contains("bash scripts/cutover/test_check_cutover_ci_exclusion_policy.sh"));
    assert!(DOC.contains(
        "cutover_ci_exclusion_policy_reason_taxonomy_version=kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "cutover_ci_exclusion_policy_reason_codes_csv=cutover_contract_lane_missing_in_ci_fast_gate,cutover_rollback_deep_lane_leaked_into_ci_fast_gate,cutover_contract_test_missing_in_ci_tools,cutover_deep_lane_test_leaked_into_ci_tools,ci_strategy_cutover_exclusion_markers_missing,ci_strategy_cutover_policy_command_missing,runtime_budget_exceeded"
    ));
    assert!(DOC.contains("cutover_rollback_deep_lane_local_only=true"));
    assert!(DOC.contains("cutover_rollback_deep_lane_excluded_from_ci_fast_gate=true"));
    assert!(DOC.contains("bash scripts/cutover/run_cutover_rollback_contract_lane.sh"));
    assert!(DOC.contains("bash scripts/cutover/run_cutover_rollback_deep_lane.sh"));
}

#[test]
fn doc_contains_invariant_fuzz_concurrency_ci_smoke_boundary_contract_markers() {
    assert!(DOC.contains("## Invariant/Fuzz/Concurrency CI Smoke Boundary Contract"));
    assert!(DOC.contains(
        "bash scripts/runtime/run_invariant_fuzz_concurrency_contract_lane.sh --output-json /tmp/invariant-fuzz-concurrency-contract-report.json"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/check_invariant_fuzz_concurrency_policy.sh --report-file /tmp/invariant-fuzz-concurrency-contract-report.json --output-json /tmp/invariant-fuzz-concurrency-policy-report.json"
    ));
    assert!(DOC.contains("bash scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh"));
    assert!(DOC.contains("bash scripts/runtime/run_concurrency_state_mutation_deep_lane.sh"));
    assert!(DOC.contains("invariant_fuzz_concurrency_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("invariant_fuzz_concurrency_local_heavy_max_seconds=900"));
    assert!(DOC.contains("invariant_fuzz_concurrency_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("invariant_fuzz_concurrency_local_heavy_execution_mode=opt_in"));
    assert!(DOC.contains("invariant_fuzz_concurrency_local_heavy_excluded_from_ci_fast_gate=true"));
}
