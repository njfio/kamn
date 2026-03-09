const PRIMITIVES_TEST_SURFACE: &str = include_str!("runtime_peer_coordination_primitives.rs");

#[test]
fn contract_runtime_peer_coordination_primitives_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_peer_lifecycle_valid_sequence_reaches_expected_states",
        "integration_runtime_peer_lifecycle_invalid_transition_fails_closed_with_reason_code",
        "integration_runtime_queue_preserves_fifo_order",
        "integration_runtime_queue_invalid_capacity_and_overflow_fail_closed",
    ] {
        assert!(PRIMITIVES_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
