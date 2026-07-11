pub(super) fn valid_create_body() -> &'static str {
    concat!(
        "{\"provider_did\":\"kamn:did:agent:task-lifecycle-provider\",",
        "\"transaction_id\":\"transaction-lifecycle-001\",",
        "\"terms_digest\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\",",
        "\"idempotency_key\":\"create-lifecycle-001\",",
        "\"description\":\"canonical lifecycle task\"}"
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

pub(super) fn assert_response(response: &str, status: &str, reason: &str) {
    assert!(response.contains(status), "{response}");
    assert!(response.contains(reason), "{response}");
}
