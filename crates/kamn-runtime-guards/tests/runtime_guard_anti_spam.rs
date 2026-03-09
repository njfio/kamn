use kamn_runtime_guards::anti_spam::{
    AntiSpamConfig, AntiSpamDecision, AntiSpamEngine, AntiSpamError, AntiSpamRejection,
    AntiSpamTelemetry,
};

fn funded_engine(config: AntiSpamConfig, sender_did: &str, deposit: u64) -> AntiSpamEngine {
    let mut engine = AntiSpamEngine::new(config).expect("anti-spam config should be valid");
    engine
        .set_deposit(sender_did, deposit)
        .expect("sender deposit should register");
    engine
}

fn evaluate(
    engine: &mut AntiSpamEngine,
    sender_did: &str,
    message_id: &str,
    now_unix: u64,
) -> AntiSpamDecision {
    engine
        .evaluate(sender_did, message_id, now_unix)
        .expect("anti-spam evaluation should succeed")
}

fn assert_rate_limit(
    engine: &mut AntiSpamEngine,
    sender_did: &str,
    message_id: &str,
    now_unix: u64,
) {
    assert_eq!(
        evaluate(engine, sender_did, message_id, now_unix),
        AntiSpamDecision::Rejected(AntiSpamRejection::RateLimitExceeded {
            limit: 2,
            observed: 2,
            window_seconds: 5,
        })
    );
}

#[test]
fn integration_runtime_guard_anti_spam_accepts_funded_sender() {
    let sender_did = "kamn:did:agent:anti-spam-funded";
    let mut engine = funded_engine(AntiSpamConfig::default(), sender_did, 25);

    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-funded-1", 100),
        AntiSpamDecision::Accepted
    );
    assert_eq!(
        engine.telemetry(),
        AntiSpamTelemetry {
            total_processed: 1,
            accepted: 1,
            rejected_insufficient_deposit: 0,
            rejected_rate_limit: 0,
            rejected_suspended: 0,
            rejected_duplicate_message: 0,
        }
    );
}

#[test]
fn integration_runtime_guard_anti_spam_rejects_duplicate_message_and_tracks_telemetry() {
    let sender_did = "kamn:did:agent:anti-spam-duplicate";
    let mut engine = funded_engine(AntiSpamConfig::default(), sender_did, 25);

    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-duplicate", 100),
        AntiSpamDecision::Accepted
    );
    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-duplicate", 101),
        AntiSpamDecision::Rejected(AntiSpamRejection::DuplicateMessageId(
            "msg-duplicate".to_owned()
        ))
    );

    let telemetry = engine.telemetry();
    assert_eq!(telemetry.total_processed, 2);
    assert_eq!(telemetry.accepted, 1);
    assert_eq!(telemetry.rejected_duplicate_message, 1);
}

#[test]
fn integration_runtime_guard_anti_spam_rejects_unfunded_sender() {
    let sender_did = "kamn:did:agent:anti-spam-unfunded";
    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid config");

    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-unfunded", 100),
        AntiSpamDecision::Rejected(AntiSpamRejection::InsufficientDeposit {
            required: AntiSpamConfig::default().minimum_sybil_deposit,
            provided: 0,
        })
    );
    assert_eq!(engine.telemetry().rejected_insufficient_deposit, 1);
}

#[test]
fn integration_runtime_guard_anti_spam_rate_limits_then_suspends_then_recovers() {
    let sender_did = "kamn:did:agent:anti-spam-rate-limit";
    let mut engine = funded_engine(
        AntiSpamConfig {
            max_messages_per_window: 2,
            window_seconds: 5,
            suspension_violation_threshold: 2,
            suspension_seconds: 10,
            ..AntiSpamConfig::default()
        },
        sender_did,
        50,
    );

    assert_initial_accepts(&mut engine, sender_did);
    assert_rate_limit(&mut engine, sender_did, "msg-rl-block-1", 102);
    assert_rate_limit(&mut engine, sender_did, "msg-rl-block-2", 103);
    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-suspended", 104),
        AntiSpamDecision::Rejected(AntiSpamRejection::SenderSuspended { until_unix: 113 })
    );
    assert_eq!(
        evaluate(&mut engine, sender_did, "msg-recovered", 120),
        AntiSpamDecision::Accepted
    );

    let telemetry = engine.telemetry();
    assert_eq!(telemetry.total_processed, 6);
    assert_eq!(telemetry.accepted, 3);
    assert_eq!(telemetry.rejected_rate_limit, 2);
    assert_eq!(telemetry.rejected_suspended, 1);
}

fn assert_initial_accepts(engine: &mut AntiSpamEngine, sender_did: &str) {
    assert_eq!(
        evaluate(engine, sender_did, "msg-rl-ok-1", 100),
        AntiSpamDecision::Accepted
    );
    assert_eq!(
        evaluate(engine, sender_did, "msg-rl-ok-2", 101),
        AntiSpamDecision::Accepted
    );
}

#[test]
fn integration_runtime_guard_anti_spam_invalid_config_and_input_fail_closed() {
    assert_eq!(
        AntiSpamEngine::new(AntiSpamConfig {
            suspension_seconds: 0,
            ..AntiSpamConfig::default()
        }),
        Err(AntiSpamError::InvalidConfig(
            "suspension_seconds must be greater than zero".to_owned()
        ))
    );

    let mut engine = AntiSpamEngine::new(AntiSpamConfig::default()).expect("valid config");
    assert_eq!(
        engine.set_deposit("did:example:anti-spam", 10),
        Err(AntiSpamError::InvalidInput(
            "sender_did must use kamn:did:agent:* format".to_owned()
        ))
    );
    assert_eq!(
        engine.evaluate("kamn:did:agent:anti-spam-input", "", 100),
        Err(AntiSpamError::InvalidInput(
            "message_id must not be empty".to_owned()
        ))
    );
}
