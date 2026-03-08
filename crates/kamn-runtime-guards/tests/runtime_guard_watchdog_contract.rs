const WATCHDOG_TEST_SURFACE: &str = include_str!("runtime_guard_watchdog.rs");

#[test]
fn contract_runtime_guard_watchdog_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_guard_watchdog_mixed_sequence_emits_expected_alerts",
        "integration_runtime_guard_watchdog_single_recipient_gossip_emits_no_alerts",
        "integration_runtime_guard_watchdog_snapshot_tracks_mixed_warning_and_critical_counts",
        "integration_runtime_guard_watchdog_invalid_config_and_input_fail_closed",
    ] {
        assert!(WATCHDOG_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
