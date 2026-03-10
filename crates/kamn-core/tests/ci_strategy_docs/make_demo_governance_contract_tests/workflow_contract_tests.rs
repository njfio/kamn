use super::support::assert_doc_contains_all;

fn core_workflow_markers() -> &'static [&'static str] {
    &[
        "make check",
        "make test",
        "make demo",
        "## Test Layering Policy Contract",
        "scripts/ci/check_test_layering_policy.py",
        "scripts/ci/test_check_test_layering_policy.sh",
        "docs/planning/test_layering_policy.md",
        "## Snapshot + Journal Durability Replay Contract",
        "docs/planning/persistence_durability_model.md",
        "cargo test -p kamn-core --lib journal",
        "channel_snapshot_journal_corrupt_tail:<line>",
        "message_lifecycle_snapshot_journal_corrupt_tail:<line>",
        "task_operation_snapshot_journal_corrupt_tail:<line>",
        "## Runtime Backpressure Enforcement Contract",
        "docs/planning/runtime_backpressure_policy.md",
        "cargo test -p kamn-core --lib backpressure",
        "cargo test -p kamn-core --lib network_fault_simulation",
        "runtime_backpressure_reject_new_enqueue",
        "runtime_backpressure_purge_stale_peer_queue",
        "## Lifecycle Property Shrinking Contract",
        "docs/planning/property_invariant_matrix.md",
        "cargo test -p kamn-core --test lifecycle_property_shrinking",
        "cargo test -p kamn-core --test lifecycle_evidence_property_matrix",
        "minimal failing prefix",
    ]
}

fn fuzz_and_runtime_markers() -> &'static [&'static str] {
    &[
        "## Coverage-Guided Parser Fuzz Contract",
        "docs/planning/fuzz_harness_budget_policy.md",
        "run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json",
        "run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json",
        "run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json",
        "run_input_mutation_coverage_guided_deep_lane.sh",
        "runtime_input_mutation_coverage_guided_deep=skipped_local_only",
        "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY=true",
        "main_tests::functional_kolme_live_retry_emits_structured_retry_markers -- --exact",
        "main_tests::functional_runtime_daemon_emits_structured_transition_markers -- --exact",
        "kolme.live.submit.retry",
        "kolme.live.finality.retry",
        "node.runtime.daemon.execute.start",
        "node.runtime.daemon.execute.complete",
        "layering_marker_missing",
        "run_localhost_signed_integration_contract_lane_tests",
        "sdk-live-localhost-integration",
        "KAMN_CI_TOOLS_FAST_MODE=true",
    ]
}

fn shell_surface_markers() -> &'static [&'static str] {
    &[
        "bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh",
        "`validate_unified_api_observability_local_heavy_live.sh --mode run --ci-fast-gate FAIL` must not appear in `.github/workflows/ci-fast-gate.yml` or `scripts/ci/test_ci_tools.sh` fast-mode block.",
        "cargo test -p kamn-core --test shell_test_surface_migration_wave1",
        "cargo test -p kamn-core --test shell_test_surface_ratio_policy",
        "legacy ingress parser drift checker contract",
        "check_legacy_ingress_parser_drift.sh --source-root crates/kamn-node/src --baseline-file fixtures/ci/legacy_ingress_parser_baseline.json --output-json /tmp/legacy-ingress-parser-drift-report.json",
        "fixtures/ci/shell_test_surface_ratio_baseline.env",
        ".ci/shell_test_surface_ratio_thresholds.env",
        "policy_status=within|waiver-applied|fail",
        "reason_codes=ratio_fail_threshold_exceeded_unwaived",
        "reason_codes=ratio_fail_threshold_waiver_applied",
        "reason_codes=legacy_ingress_parser_marker_count_increased",
        "reason_codes=legacy_ingress_parser_marker_new_file",
        "reason_codes=legacy_ingress_parser_baseline_missing",
        "reason_codes=legacy_ingress_parser_baseline_invalid",
        "run_localhost_signed_integration_contract_lane.sh",
        "scripts/ci/select_targets.sh",
        "run_kolme_version_compatibility_contract_tests=true",
        "test_run_fast_gate_native_api_parity_contract_lane.sh",
        "run_fast_gate_native_api_parity_contract_lane.sh",
        "check_fast_gate_native_api_parity_policy.py",
        "KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120",
        "test_run_continuous_runtime_commit_contract_lane.sh",
        "test_run_did_lifecycle_chain_adapter_contract_lane.sh",
        "test_run_message_proof_anchoring_contract_lane.sh",
        "test_run_managed_signer_startup_live_validation_contract_lane.sh",
        "test_validate_continuous_runtime_commit_live.sh",
        "test_validate_did_lifecycle_chain_adapter_live.sh",
        "test_validate_message_proof_anchoring_live.sh",
    ]
}

#[test]
fn doc_contains_core_make_demo_workflow_markers() {
    assert_doc_contains_all(core_workflow_markers(), "core make/demo workflow");
}

#[test]
fn doc_contains_fuzz_and_runtime_workflow_markers() {
    assert_doc_contains_all(fuzz_and_runtime_markers(), "fuzz and runtime workflow");
}

#[test]
fn doc_contains_shell_surface_workflow_markers() {
    assert_doc_contains_all(shell_surface_markers(), "shell surface workflow");
}
