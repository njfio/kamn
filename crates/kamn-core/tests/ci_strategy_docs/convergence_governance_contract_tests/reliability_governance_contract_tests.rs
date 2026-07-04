use super::super::fairness_deletion_support::assert_contains_all;
use super::super::DOC;

#[test]
fn doc_contains_panic_path_policy_checker_markers_and_remediation_parity() {
    assert_contains_all(
        DOC,
        &[
            "## Panic-Path Policy Checker Fast Lane",
            "bash scripts/ci/check_no_production_expect.sh --output-json /tmp/no-production-expect-report.json",
            "cargo clippy --workspace --lib --bins -- -D warnings -D clippy::expect_used",
            "bash scripts/ci/test_check_no_production_expect.sh",
            "kamn.ci.production-panic-replacement-reason-taxonomy.v1",
            "scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default",
            "runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv",
            "panic_path_policy_scope_root=crates/kamn-node/src",
            "panic_path_policy_production_target_scope=lib+bins",
            "panic_path_policy_test_target_exclusion=tests-benches-excluded",
            "panic_path_policy_ci_smoke_max_seconds=30",
            "panic_path_policy_remediation_steps_version=v1",
            "panic_path_policy_remediation_step_1=replace_panic_primitives_with_typed_errors",
            "panic_path_policy_remediation_step_2=rerun_checker_until_status_ok",
            "panic_path_policy_remediation_step_3=attach_reason_codes_and_evidence_outputs_to_pr",
        ],
        "panic path policy",
    );
}

#[test]
fn doc_contains_signer_quorum_go_no_go_policy_markers() {
    assert_contains_all(
        DOC,
        &[
            "signer_quorum_go_no_go_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signer-quorum-go-no-go-reason-taxonomy.v1",
            "signer_quorum_go_no_go_reason_codes_csv=runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation",
            "signer_disagreement_go_no_go_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signer-disagreement-go-no-go-reason-taxonomy.v1",
            "signer_disagreement_go_no_go_reason_codes_csv=runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_failover_attestation_previous_profile_not_approved",
            "signer_quorum_go_no_go_status=verified|drift_detected",
            "signer_quorum_go_no_go_decision=GO|NO-GO",
            "signer_disagreement_go_no_go_status=verified|disagreement_detected",
            "signer_disagreement_go_no_go_decision=GO|NO-GO",
            "python3 scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py",
        ],
        "signer quorum governance",
    );
}

#[test]
fn doc_contains_task_escrow_suite_discovery_and_parallel_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Task Escrow Suite Discovery + Parallel Boundary Contract",
            "cargo test -p kamn-core --test task_escrow_suite_modularization_contract",
            "cargo test -p kamn-core --test task_escrow_suite_discovery_parallel_contract",
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_task_escrow_suite_discovery_and_parallel_contract_markers -- --exact",
            "task_escrow_suite_discovery_contract_status=verified",
            "task_escrow_suite_discovery_expected_modules_csv=shared,task_domain,escrow_domain",
            "task_escrow_suite_parallel_seed_isolation_status=verified",
            "task_escrow_suite_parallel_case_budget_max=256",
            "task_escrow_suite_parallel_sequence_budget_max=32",
        ],
        "task escrow discovery",
    );
}

#[test]
fn doc_contains_quota_policy_checker_taxonomy_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "### Quota Policy Checker Taxonomy Contract",
            "quota_policy_checker_reason_taxonomy_version=kamn.runtime.quota-policy-reason-taxonomy.v1",
            "quota_policy_checker_reason_codes_csv=quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive,quota_limit_exceeded",
            "quota_policy_checker_fixture_schema_version=kamn.runtime.quota-policy-fixture-matrix.v1",
            "quota_policy_checker_fixture_path=fixtures/runtime/quota_policy_fixture_matrix.txt",
            "cargo test -p kamn-core --test quota_policy_checker_contract",
            "cargo test -p kamn-core --test quota_policy_fixture_parser_contract",
            "Regression: #4091",
        ],
        "quota policy",
    );
}
