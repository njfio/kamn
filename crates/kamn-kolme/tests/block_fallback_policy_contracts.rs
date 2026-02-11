use kamn_kolme::{
    parse_block_fallback_response, parse_fork_block_fallback_response,
    KolmeBlockFallbackPolicyError, KolmeBlockFallbackResponse,
};

#[test]
fn functional_parse_block_fallback_response_accepts_flat_json_shape() {
    let response = parse_block_fallback_response(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":72,\"tx_hashes\":\"ab12cd34,ffee\",\"failed_tx_hashes\":\"deadbeef\"}",
    )
    .expect("flat json fallback response should parse");
    assert_eq!(
        response,
        KolmeBlockFallbackResponse {
            provider: "kolme-fork-local".to_owned(),
            block_height: 72,
            finalized_tx_hashes: vec!["ab12cd34".to_owned(), "ffee".to_owned()],
            failed_tx_hashes: vec!["deadbeef".to_owned()],
        }
    );
}

#[test]
fn functional_parse_fork_block_fallback_response_maps_txhash_to_finalized_bucket() {
    let response =
        parse_fork_block_fallback_response("{\"txhash\":\"ab12cd34\"}", "kolme-fork-local", 42)
            .expect("fork fallback response should parse");
    assert_eq!(response.provider, "kolme-fork-local");
    assert_eq!(response.block_height, 42);
    assert_eq!(response.finalized_tx_hashes, vec!["ab12cd34".to_owned()]);
    assert!(response.failed_tx_hashes.is_empty());
}

#[test]
fn regression_issue_1751_parse_block_fallback_response_rejects_empty_hash_token() {
    // Regression: #1751
    let error = parse_block_fallback_response(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":72,\"tx_hashes\":\"ab12cd34, ,ffee\"}",
    )
    .expect_err("empty tx hash list token must fail");
    assert_eq!(
        error,
        KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "field contains empty tx hash token: tx_hashes".to_owned(),
        }
    );
}

#[test]
fn regression_issue_1751_parse_fork_block_fallback_response_rejects_empty_provider() {
    // Regression: #1751
    let error = parse_fork_block_fallback_response("{\"txhash\":\"ab12cd34\"}", " ", 42)
        .expect_err("empty provider must fail");
    assert_eq!(
        error,
        KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "provider must not be empty".to_owned(),
        }
    );
}
