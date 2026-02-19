const WATCHDOG_DOC: &str = include_str!("../../../docs/foundation/runtime-watchdog-attestation.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");

const OWNERSHIP_REASON_TAXONOMY_VERSION: &str =
    "kamn.node.runtime-test-ownership-reason-taxonomy.v1";
const OWNERSHIP_REASON_CODES_CSV: &str = "runtime_tests_shell_owner_marker_missing,runtime_tests_fragment_owner_marker_missing,runtime_tests_guard_command_marker_missing";

#[test]
fn doc_contains_node_runtime_test_ownership_taxonomy_and_markers() {
    assert!(WATCHDOG_DOC.contains("## Node Runtime Test-Surface Ownership Mapping"));
    assert!(WATCHDOG_DOC.contains(
        format!(
            "node_runtime_test_ownership_reason_taxonomy_version={}",
            OWNERSHIP_REASON_TAXONOMY_VERSION
        )
        .as_str()
    ));
    assert!(WATCHDOG_DOC.contains(
        format!(
            "node_runtime_test_ownership_reason_codes_csv={}",
            OWNERSHIP_REASON_CODES_CSV
        )
        .as_str()
    ));
    assert!(WATCHDOG_DOC.contains("node_runtime_test_ownership_status=verified"));
    assert!(WATCHDOG_DOC.contains("crates/kamn-node/src/main_tests/runtime_tests.rs"));
    assert!(WATCHDOG_DOC
        .contains("crates/kamn-node/src/main_tests/runtime_tests/arg_and_signer_policy_tests.rs"));
    assert!(WATCHDOG_DOC
        .contains("crates/kamn-node/src/main_tests/runtime_tests/logging_and_bootstrap_tests.rs"));
    assert!(WATCHDOG_DOC.contains(
        "crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs"
    ));
    assert!(WATCHDOG_DOC.contains(
        "crates/kamn-node/src/main_tests/runtime_tests/full_supervisor_and_shutdown_tests.rs"
    ));
    assert!(WATCHDOG_DOC.contains(
        "crates/kamn-node/src/main_tests/runtime_tests/profile_and_config_layering_tests.rs"
    ));
    assert!(WATCHDOG_DOC
        .contains("crates/kamn-node/src/main_tests/runtime_tests/kolme_live_execution_tests.rs"));
    assert!(WATCHDOG_DOC.contains("cargo test -p kamn-node --test main_module_extraction_contract main_module_extraction_contract_runtime_tests_decomposition_shell_markers_remain_stable -- --exact"));
}

#[test]
fn ci_strategy_contains_node_runtime_test_ownership_guard_command_marker() {
    assert!(
        CI_STRATEGY_DOC.contains(
            "cargo test -p kamn-core --test node_test_surface_ownership_docs -- --nocapture"
        ),
        "reason_taxonomy_version={} reason_codes_csv={} reason_code=runtime_tests_guard_command_marker_missing",
        OWNERSHIP_REASON_TAXONOMY_VERSION,
        OWNERSHIP_REASON_CODES_CSV
    );
}
