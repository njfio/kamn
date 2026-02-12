use kamn_kolme::{
    is_valid_block_fallback_base_url_input, is_valid_block_fallback_lookup_budget,
    is_valid_block_fallback_provider_input, normalize_block_fallback_constructor_inputs,
    parse_block_fallback_response, parse_fork_block_fallback_response,
    parse_provider_block_fallback_response, KolmeBlockFallbackPolicyError,
    KolmeBlockFallbackResponse,
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
fn functional_parse_provider_block_fallback_response_accepts_canonical_payload_shape() {
    let response = parse_provider_block_fallback_response(
        "{\"provider\":\"kolme-fork-local\",\"block_height\":72,\"tx_hashes\":\"ab12cd34,ffee\"}",
        "kolme-fork-local",
        72,
    )
    .expect("provider-aware canonical fallback response should parse");
    assert_eq!(
        response,
        KolmeBlockFallbackResponse {
            provider: "kolme-fork-local".to_owned(),
            block_height: 72,
            finalized_tx_hashes: vec!["ab12cd34".to_owned(), "ffee".to_owned()],
            failed_tx_hashes: Vec::new(),
        }
    );
}

#[test]
fn functional_parse_provider_block_fallback_response_falls_back_to_fork_shape() {
    let response =
        parse_provider_block_fallback_response("{\"txhash\":\"ab12cd34\"}", "kolme-fork-local", 42)
            .expect("provider-aware parser should fall back to fork payload shape");
    assert_eq!(response.provider, "kolme-fork-local");
    assert_eq!(response.block_height, 42);
    assert_eq!(response.finalized_tx_hashes, vec!["ab12cd34".to_owned()]);
    assert!(response.failed_tx_hashes.is_empty());
}

#[test]
fn regression_issue_1836_parse_provider_block_fallback_response_fails_closed_when_both_shapes_invalid(
) {
    // Regression: #1836
    let error = parse_provider_block_fallback_response("{}", "kolme-fork-local", 42)
        .expect_err("provider-aware parser must fail closed when payload is invalid");
    assert_eq!(
        error,
        KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "missing required field: txhash".to_owned(),
        }
    );
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

#[test]
fn functional_block_fallback_policy_accepts_valid_constructor_guard_inputs() {
    assert!(is_valid_block_fallback_base_url_input(
        "https://kolme.example"
    ));
    assert!(is_valid_block_fallback_provider_input("kolme-fork-local"));
    assert!(is_valid_block_fallback_lookup_budget(5));
}

#[test]
fn functional_block_fallback_policy_normalizes_constructor_inputs() {
    let normalized = normalize_block_fallback_constructor_inputs(
        "  https://kolme.example  ",
        "  /block/{height}  ",
        "  kolme-fork-local  ",
    );
    assert_eq!(
        normalized,
        (
            "https://kolme.example".to_owned(),
            "/block/{height}".to_owned(),
            "kolme-fork-local".to_owned(),
        )
    );
}

#[test]
fn regression_issue_1866_block_fallback_policy_rejects_invalid_constructor_guard_inputs() {
    // Regression: #1866
    assert!(!is_valid_block_fallback_base_url_input(" "));
    assert!(!is_valid_block_fallback_provider_input(""));
    assert!(!is_valid_block_fallback_lookup_budget(0));
}

#[test]
fn regression_issue_1922_block_fallback_policy_trims_constructor_input_whitespace() {
    // Regression: #1922
    let normalized = normalize_block_fallback_constructor_inputs(
        "\nhttps://kolme.example\n",
        "\n/block/{height}\n",
        "\nkolme-fork-local\n",
    );
    assert_eq!(
        normalized,
        (
            "https://kolme.example".to_owned(),
            "/block/{height}".to_owned(),
            "kolme-fork-local".to_owned(),
        )
    );
}
