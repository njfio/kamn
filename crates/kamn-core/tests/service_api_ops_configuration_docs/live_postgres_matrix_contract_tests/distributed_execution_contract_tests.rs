use super::*;

#[test]
fn service_api_ops_configuration_contains_daemon_tests_live_postgres_fixture_decomposition_markers()
{
    assert_doc_contains_all(&["### Daemon Tests Live-Postgres Fixture Decomposition Contracts (Issue #5402, #5418, #5420)", "daemon_tests_live_postgres_fixture_module_path=crates/kamn-node/src/main_tests/daemon_tests/live_postgres_fixtures.rs", "daemon_tests_live_postgres_fixture_phase1_target_max_lines=4300", "daemon_tests_live_postgres_fixture_phase2_module_path=crates/kamn-node/src/main_tests/daemon_tests/live_postgres_topology_contract_tests.rs", "daemon_tests_live_postgres_fixture_phase2_target_max_lines=2200", "daemon_tests_live_postgres_fixture_phase3_runtime_module_path=crates/kamn-node/src/main_tests/daemon_tests/runtime_contract_tests.rs", "daemon_tests_live_postgres_fixture_phase3_matrix_module_path=crates/kamn-node/src/main_tests/daemon_tests/live_postgres_matrix_contract_tests.rs", "daemon_tests_live_postgres_fixture_phase3_root_target_max_lines=300", "daemon_tests_live_postgres_fixture_test_path_contract=main_tests::daemon_tests::path_prefix_must_remain_stable_after_fixture_extraction", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_contract_is_canonical -- --exact", "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_daemon_tests_live_postgres_fixture_decomposition_markers -- --exact", "Regression: #5402, #5418, #5420"]);
}
#[test]
fn service_api_ops_configuration_contains_multi_host_batched_coherence_bundle_markers() {
    assert!(DOC.contains(
        "### Multi-Host Daemon Live-Postgres Batched Coherence Bundle Map (Issue #5422)"
    ));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b01=runtime_matrix_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b02=parallel_lane_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b03=topology_mapping_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b04=topology_coherence_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b05=fingerprint_stability_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_b06=multi_host_execution_bundle"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_issue_ceiling=8"));
    assert!(DOC.contains("daemon_live_postgres_coherence_bundle_issue_floor=5"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_multi_host_batched_coherence_bundle_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #5422"));
}

#[test]
fn service_api_ops_configuration_contains_multi_host_distributed_execution_bundle_markers() {
    assert_doc_contains_all(&["### Multi-Host Distributed Execution Lane Contracts (Issue #5422)", "daemon_live_postgres_multi_host_execution_reason_taxonomy_version=kamn.runtime.daemon.phase6-live-postgres.multi-host-execution.reason-taxonomy.v1", "daemon_live_postgres_multi_host_execution_prerequisite_env_keys_csv=KAMN_TEST_POSTGRES_URL|DATABASE_URL,KAMN_TEST_LIVE_POSTGRES_DISTRIBUTED_HOSTS", "daemon_live_postgres_multi_host_execution_prerequisite_reason_codes_csv=live_postgres_multi_host_prerequisites_ready,live_postgres_multi_host_prerequisites_missing,live_postgres_multi_host_host_pair_invalid", "daemon_live_postgres_multi_host_execution_bundle_selector_prefix=main_tests::daemon_tests::", "daemon_live_postgres_multi_host_execution_bundle_selector_rows_csv=b01_runtime_matrix_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_matrix_reasons_are_stable_across_repeated_runs,b02_parallel_lane_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_fingerprint_schema_is_stable,b03_topology_mapping_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_scope_is_stable,b04_topology_coherence_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_id_bundle_coherence_is_stable,b05_fingerprint_stability_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_parallel_lane_topology_host_mode_host_pair_lane_set_lane_fingerprint_hash_order_normalization_digest_is_stable,b06_multi_host_execution_bundle->main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable", "daemon_live_postgres_multi_host_execution_projection_digest_fnv1a64_hex=25b9729eaeb44fe9", "daemon_live_postgres_multi_host_execution_contract=prerequisite_gate->distributed_label_projection->fingerprint_digest_stability", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_prerequisite_guard_contract_is_canonical -- --exact", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::functional_runtime_daemon_live_postgres_validation_slice_multi_host_execution_bundle_selector_rows_are_canonical -- --exact", "cargo test -p kamn-node --bin kamn-node main_tests::daemon_tests::integration_runtime_daemon_phase6_live_postgres_validation_slice_multi_host_execution_bundle_is_stable -- --exact", "cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_multi_host_distributed_execution_bundle_markers -- --exact", "Regression: #5422"]);
}
