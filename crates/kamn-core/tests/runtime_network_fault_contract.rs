const NETWORK_FAULT_TEST_SURFACE: &str = include_str!("runtime_network_fault_integration.rs");

#[test]
fn contract_runtime_network_fault_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_network_fault_valid_simulation_returns_expected_report",
        "integration_runtime_network_fault_invalid_inputs_fail_closed_with_reason_codes",
        "integration_runtime_network_fault_daemon_helper_matches_direct_simulation",
    ] {
        assert!(NETWORK_FAULT_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
