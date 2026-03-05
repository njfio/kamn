use super::*;

const REQUEST_PROFILE_KEY_ID: &str = "secure:key-ops-1";
const REQUEST_PROFILE_SENDER: &str = "agent-a";
const REQUEST_PROFILE_NONCE: u64 = 1;
const REQUEST_PROFILE_PAYLOAD: &str = "payload-1";
const REQUEST_PROFILE_PREFIX: &str = "sig:secp256k1:baseline-v2:";
const REQUEST_TRANSACTION_KEY_ID: &str = "secure:key-ops-3";
const REQUEST_PLACEHOLDER_SIGNATURE: &str = "sig:placeholder";

fn signer_profile_request() -> SigningRequest {
    SigningRequest::new(
        REQUEST_PROFILE_KEY_ID,
        REQUEST_PROFILE_SENDER,
        REQUEST_PROFILE_NONCE,
        REQUEST_PROFILE_PAYLOAD,
        GENESIS_STATE_HASH,
    )
    .expect("request should be valid")
}

fn assert_signature_includes_profile_prefix(message: &str) {
    with_default_signer_key_env(|| {
        let router = SignerBackendRouter::default();
        let request = signer_profile_request();
        let signed = router
            .sign_with_secure_fallback(&request)
            .expect("signature should be produced");
        assert!(signed.signature.starts_with(REQUEST_PROFILE_PREFIX), "{message}");
    });
}

pub(super) fn run_for_transaction_rejects_empty_transaction_id() {
    let tx = BaselineTransaction {
        id: String::new(),
        sender: REQUEST_PROFILE_SENDER.to_owned(),
        nonce: REQUEST_PROFILE_NONCE,
        payload: REQUEST_PROFILE_PAYLOAD.to_owned(),
        state_hash: GENESIS_STATE_HASH.to_owned(),
        signature: REQUEST_PLACEHOLDER_SIGNATURE.to_owned(),
    };
    assert_eq!(
        SigningRequest::for_transaction(REQUEST_TRANSACTION_KEY_ID, &tx),
        Err(SignerBackendError::EmptyField("transaction_id"))
    );
}

pub(super) fn run_regression_signing_request_matches_canonical_signature_profile() {
    // Regression: #400
    assert_signature_includes_profile_prefix(
        "default signer path must emit cryptographic baseline-v2 signatures",
    );
}

pub(super) fn run_regression_signatures_include_profile_identifier_segment() {
    // Regression: #404
    assert_signature_includes_profile_prefix("signature should include profile identifier segment");
}
