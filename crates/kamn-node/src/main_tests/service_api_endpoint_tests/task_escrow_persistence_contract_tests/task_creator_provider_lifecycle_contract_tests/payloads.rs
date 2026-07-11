pub(super) fn valid_create_body(provider_did: &str) -> String {
    format!(
        r#"{{"provider_did":"{provider_did}","transaction_id":"transaction-lifecycle-001","terms_digest":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","idempotency_key":"create-lifecycle-001","description":"canonical lifecycle task"}}"#
    )
}

pub(super) fn retry_body(idempotency_key: &str) -> &str {
    match idempotency_key {
        "accept-1" => r#"{"idempotency_key":"accept-1"}"#,
        "wrong-provider-accept" => r#"{"idempotency_key":"wrong-provider-accept"}"#,
        "complete-early" => {
            r#"{"idempotency_key":"complete-early","completion_evidence_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}"#
        }
        "accept-valid" => r#"{"idempotency_key":"accept-valid"}"#,
        _ => r#"{"idempotency_key":"complete-missing-evidence"}"#,
    }
}

pub(super) fn completion_body(idempotency_key: &str) -> String {
    format!(
        r#"{{"idempotency_key":"{idempotency_key}","completion_evidence_digest":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}}"#
    )
}

pub(super) fn assert_response(response: &str, status: &str, reason: &str) {
    assert!(response.contains(status), "{response}");
    assert!(response.contains(reason), "{response}");
}
