use kamn_core::{
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse, KolmeRuntimeCommitError, KolmeRuntimeCommitProviderError,
};

#[test]
fn unit_nonce_request_rejects_empty_pubkey() {
    assert_eq!(
        KolmeApiNextNonceRequest::new(""),
        Err(KolmeRuntimeCommitError::InvalidRequest {
            field: "pubkey",
            reason: "must not be empty",
        })
    );
}

#[test]
fn unit_nonce_request_builds_encoded_query_path() {
    let request =
        KolmeApiNextNonceRequest::new("pub:key/with space").expect("request should build");
    assert_eq!(
        request.query_path("/get-next-nonce"),
        "/get-next-nonce?pubkey=pub%3Akey%2Fwith%20space"
    );
}

#[test]
fn functional_broadcast_request_serializes_canonical_json_payload() {
    let request = KolmeApiBroadcastRequest::new("{\"messages\":[]}", "0xabc123", 1)
        .expect("broadcast request should build");
    assert_eq!(
        request.to_json_payload(),
        "{\"message\":\"{\\\"messages\\\":[]}\",\"signature\":\"0xabc123\",\"recovery_id\":1}"
    );
}

#[test]
fn functional_nonce_response_parses_numeric_next_nonce() {
    let response =
        KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":12,\"account_id\":\"acct-1\"}")
            .expect("nonce response should parse");
    assert_eq!(response.next_nonce, 12);
    assert_eq!(response.account_id, Some("acct-1".to_owned()));
}

#[test]
fn functional_nonce_response_accepts_null_account_id() {
    let response = KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":7,\"account_id\":null}")
        .expect("nonce response with null account should parse");
    assert_eq!(response.next_nonce, 7);
    assert_eq!(response.account_id, None);
}

#[test]
fn functional_broadcast_response_parses_txhash() {
    let response = KolmeApiBroadcastResponse::parse_json("{\"txhash\":\"abc123\"}")
        .expect("broadcast response should parse");
    assert_eq!(response.txhash, "abc123");
}

#[test]
fn regression_nonce_response_rejects_non_positive_next_nonce() {
    assert_eq!(
        KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":0,\"account_id\":null}"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "next_nonce must be positive".to_owned(),
        })
    );
}

#[test]
fn regression_broadcast_response_rejects_missing_txhash() {
    assert_eq!(
        KolmeApiBroadcastResponse::parse_json("{\"status\":\"ok\"}"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing required field: txhash".to_owned(),
        })
    );
}

#[test]
fn integration_nonce_and_broadcast_codec_contracts_are_compatible() {
    let nonce_request =
        KolmeApiNextNonceRequest::new("02abcd").expect("nonce request should build");
    assert_eq!(
        nonce_request.query_path("/get-next-nonce"),
        "/get-next-nonce?pubkey=02abcd"
    );

    let nonce_response =
        KolmeApiNextNonceResponse::parse_json("{\"next_nonce\":42,\"account_id\":\"acc-42\"}")
            .expect("nonce response should parse");
    assert_eq!(nonce_response.next_nonce, 42);

    let broadcast_request =
        KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 0).expect("request should build");
    assert_eq!(
        broadcast_request.to_json_payload(),
        "{\"message\":\"{\\\"nonce\\\":42}\",\"signature\":\"sig-42\",\"recovery_id\":0}"
    );

    let broadcast_response = KolmeApiBroadcastResponse::parse_json("{\"txhash\":\"tx-42\"}")
        .expect("response should parse");
    assert_eq!(broadcast_response.txhash, "tx-42");
}
