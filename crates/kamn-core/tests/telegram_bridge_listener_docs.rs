const DOC: &str = include_str!("../../../docs/foundation/telegram-bridge-listener-validation.md");

#[test]
fn doc_contains_telegram_bridge_scope_and_listener_contracts() {
    assert!(DOC.contains("# Telegram Bridge Listener-Validated Inbound Flow"));
    assert!(DOC.contains("TelegramBridgeConfig"));
    assert!(DOC.contains("process_inbound_to_envelope(...)"));
    assert!(DOC.contains("listener DID must be authorized"));
    assert!(DOC.contains("webhook token must match configured Telegram auth token"));
    assert!(DOC.contains("checkpoint must be monotonic per `external_channel_id`"));
}

#[test]
fn doc_contains_bridge_replay_subset_validation_lane() {
    assert!(DOC.contains("scripts/bridge/run_bridge_replay_matrix.sh"));
    assert!(DOC.contains("--suites bridge_adapter,telegram_bridge"));
    assert!(DOC.contains("bridge_replay_suites"));
    assert!(DOC.contains("run_telegram_ingress_contract_lane.sh"));
    assert!(DOC.contains("run_telegram_ingress_deep_lane.sh"));
}

#[test]
fn regression_requires_replay_fixture_reference() {
    // Regression: #587
    assert!(DOC.contains("duplicate replay"));
    assert!(DOC.contains("Regression: #587"));
}

#[test]
fn regression_requires_forged_webhook_and_checkpoint_rejection_rule() {
    // Regression: #621
    assert!(DOC.contains(
        "forged webhook tokens and replayed/out-of-order checkpoints are rejected (`Regression: #621`)."
    ));
}
