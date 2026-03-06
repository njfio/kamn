use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 5] = [
    "fn integration_decrypt_rejects_invalid_ciphertext_hex()",
    "fn integration_decrypt_rejects_invalid_auth_tag_hex_as_integrity_failure()",
    "fn integration_decrypt_rejects_invalid_utf8_plaintext_output()",
    "fn contract_encrypt_maps_aead_failure_to_encryption_failed()",
    "fn contract_hkdf_failure_maps_to_key_derivation_failed()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_failure_path_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-crypto/tests/direct_message_crypto_failure_paths.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "direct-message failure-path target should contain marker: {marker}"
        );
    }
}
