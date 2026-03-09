const DURABLE_GUARD_STORE_TEST_SURFACE: &str = include_str!("durable_guard_store_integration.rs");

#[test]
fn contract_durable_guard_store_surface_exists_with_expected_markers() {
    for marker in [
        "integration_durable_guard_bundle_capture_and_restore_reproduces_guard_state",
        "integration_durable_guard_in_memory_store_round_trips_bundle",
        "integration_durable_guard_file_store_round_trips_bundle_from_disk",
        "integration_durable_guard_store_invalid_schema_and_payload_fail_closed",
    ] {
        assert!(
            DURABLE_GUARD_STORE_TEST_SURFACE.contains(marker),
            "missing marker: {marker}"
        );
    }
}
