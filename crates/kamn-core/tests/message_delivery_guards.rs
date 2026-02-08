use kamn_core::{
    DeliveryFailureCode, DeliveryGuardInput, DeliveryValidationResult, MessageDeliveryGuards,
};

fn input(message_id: &str, nonce: u64, received_at: &str) -> DeliveryGuardInput {
    DeliveryGuardInput {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipient: "kamn:did:agent:recipient-1".to_owned(),
        nonce,
        created: "2026-02-07T20:15:30.123Z".to_owned(),
        expires: "2026-02-07T20:45:30.123Z".to_owned(),
        received_at: received_at.to_owned(),
    }
}

#[test]
fn accepts_valid_delivery_and_advances_nonce() {
    let mut guards = MessageDeliveryGuards::new();

    assert_eq!(
        guards.validate(input("urn:uuid:msg-1", 1, "2026-02-07T20:20:30.123Z")),
        DeliveryValidationResult::Accepted
    );
    assert_eq!(guards.expected_nonce("kamn:did:agent:sender-1"), 2);
}

#[test]
fn rejects_nonce_out_of_sequence_with_failed_delivery_notice() {
    let mut guards = MessageDeliveryGuards::new();

    let result = guards.validate(input("urn:uuid:msg-2", 2, "2026-02-07T20:20:30.123Z"));
    match result {
        DeliveryValidationResult::Rejected(notice) => {
            assert_eq!(
                notice.code,
                DeliveryFailureCode::NonceOutOfSequence {
                    expected: 1,
                    found: 2,
                }
            );
            assert!(
                notice
                    .signature
                    .starts_with("notice:urn:uuid:msg-2:nonce_out_of_sequence"),
                "signature should include deterministic notice prefix"
            );
        }
        DeliveryValidationResult::Accepted => panic!("expected rejection"),
    }
}

#[test]
fn rejects_replay_message_id() {
    let mut guards = MessageDeliveryGuards::new();
    assert_eq!(
        guards.validate(input("urn:uuid:msg-3", 1, "2026-02-07T20:20:30.123Z")),
        DeliveryValidationResult::Accepted
    );

    let result = guards.validate(input("urn:uuid:msg-3", 2, "2026-02-07T20:25:30.123Z"));
    match result {
        DeliveryValidationResult::Rejected(notice) => {
            assert_eq!(notice.code, DeliveryFailureCode::Replay);
        }
        DeliveryValidationResult::Accepted => panic!("expected replay rejection"),
    }
}

#[test]
fn rejects_expired_message() {
    let mut guards = MessageDeliveryGuards::new();
    let result = guards.validate(input("urn:uuid:msg-4", 1, "2026-02-07T20:50:30.123Z"));
    match result {
        DeliveryValidationResult::Rejected(notice) => {
            assert_eq!(notice.code, DeliveryFailureCode::Expired);
        }
        DeliveryValidationResult::Accepted => panic!("expected expiry rejection"),
    }
}

#[test]
fn rejected_delivery_does_not_advance_nonce() {
    let mut guards = MessageDeliveryGuards::new();
    let _ = guards.validate(input("urn:uuid:msg-5", 2, "2026-02-07T20:20:30.123Z"));

    // Regression: #117
    assert_eq!(
        guards.validate(input("urn:uuid:msg-6", 1, "2026-02-07T20:21:30.123Z")),
        DeliveryValidationResult::Accepted
    );
    assert_eq!(guards.expected_nonce("kamn:did:agent:sender-1"), 2);
}
