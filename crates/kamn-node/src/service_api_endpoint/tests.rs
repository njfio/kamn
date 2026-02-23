use super::{
    auth::map_anti_spam_rejection_to_reasoned_error, AntiSpamRejection,
    REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
    REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
    REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED, REASON_CODE_INGRESS_SENDER_SUSPENDED,
};

#[test]
fn anti_spam_rate_limit_rejection_maps_to_sender_rate_limit_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::RateLimitExceeded {
        limit: 3,
        observed: 3,
        window_seconds: 5,
    });
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED
    );
    assert!(error.message.contains("observed=3"));
}

#[test]
fn anti_spam_sender_suspension_maps_to_sender_suspended_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::SenderSuspended {
        until_unix: 123_456,
    });
    assert_eq!(error.reason_code, REASON_CODE_INGRESS_SENDER_SUSPENDED);
    assert!(error.message.contains("123456"));
}

#[test]
fn anti_spam_insufficient_deposit_maps_to_sender_deposit_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::InsufficientDeposit {
        required: 9,
        provided: 4,
    });
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT
    );
    assert!(error.message.contains("required=9"));
    assert!(error.message.contains("provided=4"));
}

#[test]
fn anti_spam_duplicate_message_maps_to_sender_duplicate_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::DuplicateMessageId(
        "message-1".to_owned(),
    ));
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID
    );
    assert!(error.message.contains("message-1"));
}
