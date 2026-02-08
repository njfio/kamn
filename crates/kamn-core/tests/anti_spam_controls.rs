use kamn_core::{
    AntiSpamConfig, AntiSpamDecision, AntiSpamEngine, AntiSpamError, AntiSpamRejection,
};

#[test]
fn sufficient_deposit_and_within_limit_is_accepted() {
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("config should build");
    engine
        .set_deposit("kamn:did:agent:sender-1", 100)
        .expect("deposit registration should succeed");

    let decision = engine
        .evaluate("kamn:did:agent:sender-1", "msg-1", 1)
        .expect("evaluation should succeed");

    assert_eq!(decision, AntiSpamDecision::Accepted);
}

#[test]
fn insufficient_deposit_is_rejected() {
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("config should build");
    engine
        .set_deposit("kamn:did:agent:sender-1", 5)
        .expect("deposit registration should succeed");

    let decision = engine
        .evaluate("kamn:did:agent:sender-1", "msg-1", 1)
        .expect("evaluation should succeed");

    assert_eq!(
        decision,
        AntiSpamDecision::Rejected(AntiSpamRejection::InsufficientDeposit {
            required: 10,
            provided: 5,
        })
    );
}

#[test]
fn rate_limit_overrun_is_rejected_and_suspends_after_threshold() {
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("config should build");
    engine
        .set_deposit("kamn:did:agent:sender-1", 100)
        .expect("deposit registration should succeed");

    for index in 0..3 {
        let message_id = format!("msg-ok-{index}");
        let decision = engine
            .evaluate("kamn:did:agent:sender-1", &message_id, 10 + index)
            .expect("evaluation should succeed");
        assert_eq!(decision, AntiSpamDecision::Accepted);
    }

    for index in 0..2 {
        let message_id = format!("msg-spam-{index}");
        let decision = engine
            .evaluate("kamn:did:agent:sender-1", &message_id, 13 + index)
            .expect("evaluation should succeed");
        assert!(matches!(
            decision,
            AntiSpamDecision::Rejected(AntiSpamRejection::RateLimitExceeded { .. })
        ));
    }

    let suspended = engine
        .evaluate("kamn:did:agent:sender-1", "msg-suspended", 15)
        .expect("evaluation should succeed");
    assert!(matches!(
        suspended,
        AntiSpamDecision::Rejected(AntiSpamRejection::SenderSuspended { .. })
    ));
}

#[test]
fn integration_telemetry_tracks_rejection_categories() {
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("config should build");
    engine
        .set_deposit("kamn:did:agent:sender-1", 8)
        .expect("deposit registration should succeed");

    let _ = engine
        .evaluate("kamn:did:agent:sender-1", "msg-1", 1)
        .expect("evaluation should succeed");
    let _ = engine
        .evaluate("kamn:did:agent:sender-1", "msg-1", 2)
        .expect("evaluation should succeed");

    let telemetry = engine.telemetry();
    assert_eq!(telemetry.total_processed, 2);
    assert_eq!(telemetry.accepted, 0);
    assert_eq!(telemetry.rejected_insufficient_deposit, 1);
    assert_eq!(telemetry.rejected_duplicate_message, 1);
}

#[test]
fn regression_deposit_equal_to_minimum_is_accepted() {
    // Regression: #186
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("config should build");
    engine
        .set_deposit("kamn:did:agent:sender-1", 10)
        .expect("deposit registration should succeed");

    let decision = engine
        .evaluate("kamn:did:agent:sender-1", "msg-eq-min", 1)
        .expect("evaluation should succeed");
    assert_eq!(decision, AntiSpamDecision::Accepted);
}

#[test]
fn rejects_invalid_configuration() {
    assert_eq!(
        AntiSpamEngine::new(AntiSpamConfig {
            max_messages_per_window: 0,
            window_seconds: 10,
            minimum_sybil_deposit: 10,
            suspension_violation_threshold: 2,
            suspension_seconds: 60,
        }),
        Err(AntiSpamError::InvalidConfig(
            "max_messages_per_window must be greater than zero".to_owned()
        ))
    );
}
