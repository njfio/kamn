const SOURCE: &str = include_str!("runtime_guard_anti_spam.rs");

#[test]
fn spec_c01_runtime_guard_anti_spam_surface_exists_with_expected_cases() {
    assert!(SOURCE.contains("fn integration_runtime_guard_anti_spam_accepts_funded_sender()"));
    assert!(SOURCE.contains(
        "fn integration_runtime_guard_anti_spam_rejects_duplicate_message_and_tracks_telemetry()"
    ));
    assert!(SOURCE.contains("fn integration_runtime_guard_anti_spam_rejects_unfunded_sender()"));
    assert!(SOURCE.contains(
        "fn integration_runtime_guard_anti_spam_rate_limits_then_suspends_then_recovers()"
    ));
    assert!(SOURCE
        .contains("fn integration_runtime_guard_anti_spam_invalid_config_and_input_fail_closed()"));
}
