const MESSAGE_DELIVERY_TEST_SURFACE: &str = include_str!("runtime_guard_message_delivery.rs");

#[test]
fn contract_runtime_guard_message_delivery_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_guard_message_delivery_accepts_first_message_and_advances_nonce",
        "integration_runtime_guard_message_delivery_rejects_replay_and_nonce_regression",
        "integration_runtime_guard_message_delivery_snapshot_roundtrip_restores_replay_state",
        "integration_runtime_guard_message_delivery_invalid_snapshot_fails_closed",
    ] {
        assert!(MESSAGE_DELIVERY_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
