use super::{
    classify_nonce_retry_category, deterministic_nonce_retry_backoff_millis,
    KolmeRuntimeCommitProviderError,
};

#[test]
fn unit_nonce_retry_classifier_marks_transient_provider_errors() {
    assert_eq!(
        classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::Timeout),
        Some("timeout")
    );
    assert_eq!(
        classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::Unavailable {
            reason: "network unavailable".to_owned(),
        }),
        Some("unavailable")
    );
    assert_eq!(
        classify_nonce_retry_category(&KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing next_nonce".to_owned(),
        }),
        None
    );
}

#[test]
fn unit_nonce_retry_backoff_policy_is_deterministic_and_bounded() {
    assert_eq!(deterministic_nonce_retry_backoff_millis(1), 10);
    assert_eq!(deterministic_nonce_retry_backoff_millis(2), 20);
    assert_eq!(deterministic_nonce_retry_backoff_millis(3), 40);
    assert_eq!(deterministic_nonce_retry_backoff_millis(8), 40);
}
