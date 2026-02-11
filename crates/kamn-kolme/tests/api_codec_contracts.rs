use kamn_kolme::{
    validate_direct_signed_transaction_message, KolmeApiBroadcastRequest,
    KolmeApiBroadcastResponse, KolmeApiCodecError, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse,
};

#[test]
fn functional_nonce_request_query_path_percent_encodes_pubkey() {
    let request = KolmeApiNextNonceRequest::new("validator:pub/key").expect("request should build");
    assert_eq!(
        request.query_path("/get-next-nonce"),
        "/get-next-nonce?pubkey=validator%3Apub%2Fkey"
    );
}

#[test]
fn functional_broadcast_request_serializes_canonical_json_order() {
    let request =
        KolmeApiBroadcastRequest::new("msg-payload", "sig-value", 1).expect("request should build");
    assert_eq!(
        request.to_json_payload(),
        "{\"message\":\"msg-payload\",\"signature\":\"sig-value\",\"recovery_id\":1}"
    );
}

#[test]
fn regression_issue_1727_nonce_and_broadcast_parsing_fails_closed() {
    // Regression: #1727
    let nonce_error = KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":0}")
        .expect_err("zero nonce must fail");
    assert_eq!(
        nonce_error,
        KolmeApiCodecError::MalformedResponse {
            reason: "next_nonce must be positive".to_owned(),
        }
    );

    let broadcast_error = KolmeApiBroadcastResponse::parse_json("{\"txhash\":\"\"}")
        .expect_err("empty txhash must fail");
    assert_eq!(
        broadcast_error,
        KolmeApiCodecError::MalformedResponse {
            reason: "field must not be empty: txhash".to_owned(),
        }
    );
}

#[test]
fn functional_validate_direct_signed_transaction_message_accepts_required_shape() {
    validate_direct_signed_transaction_message(
        r#"{"pubkey":"ed25519:abc","nonce":42,"created":"2026-02-11T00:00:00Z","messages":[{"kind":"transfer"}]}"#,
    )
    .expect("message shape should validate");
}

#[test]
fn regression_issue_1737_direct_signed_message_validation_fails_closed() {
    // Regression: #1737
    let error = validate_direct_signed_transaction_message(
        r#"{"pubkey":"ed25519:abc","nonce":42,"created":"2026-02-11T00:00:00Z","messages":"not-array"}"#,
    )
    .expect_err("non-array messages field must fail");
    assert_eq!(
        error,
        KolmeApiCodecError::MalformedResponse {
            reason: "direct signed payload message field is invalid: messages".to_owned(),
        }
    );
}
