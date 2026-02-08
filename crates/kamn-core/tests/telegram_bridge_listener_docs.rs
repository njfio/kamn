const DOC: &str = include_str!("../../../docs/foundation/telegram-bridge-listener-validation.md");

#[test]
fn doc_contains_telegram_bridge_scope_and_listener_contracts() {
    assert!(DOC.contains("# Telegram Bridge Listener-Validated Inbound Flow"));
    assert!(DOC.contains("TelegramBridgeConfig"));
    assert!(DOC.contains("process_inbound_to_envelope(...)"));
    assert!(DOC.contains("listener DID must be authorized"));
}

#[test]
fn doc_contains_bridge_replay_subset_validation_lane() {
    assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
    assert!(DOC.contains("--suites bridge_adapter,telegram_bridge"));
    assert!(DOC.contains("bridge_replay_suites"));
}

#[test]
fn regression_requires_replay_fixture_reference() {
    // Regression: #587
    assert!(DOC.contains("duplicate replay"));
    assert!(DOC.contains("Regression: #587"));
}
