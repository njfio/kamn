const SOURCE: &str = include_str!("direct_message_crypto_input_nonce_failures.rs");
const REQUIRED_MARKERS: [&str; 4] = [
    "fn integration_encrypt_rejects_empty_plaintext_payload()",
    "fn integration_encrypt_rejects_zero_nonce()",
    "fn integration_decrypt_rejects_zero_nonce_ciphertext()",
    "fn integration_encrypt_rejects_nonce_reuse()",
];

#[test]
fn spec_c01_input_nonce_failure_target_exists_with_required_markers() {
    for marker in REQUIRED_MARKERS {
        assert!(
            SOURCE.contains(marker),
            "direct_message_crypto_input_nonce_failures.rs must contain marker: {marker}"
        );
    }
}
