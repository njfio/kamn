use std::fs;

const ROOT: &str = "src/direct_message_crypto.rs";
const MAX_ROOT_LINES: usize = 180;
const REQUIRED_FILES: &[&str] = &[
    "src/direct_message_crypto/models.rs",
    "src/direct_message_crypto/errors.rs",
    "src/direct_message_crypto/engine.rs",
    "src/direct_message_crypto/key_agreement.rs",
    "src/direct_message_crypto/cipher.rs",
    "src/direct_message_crypto/validation.rs",
    "src/direct_message_crypto/encoding.rs",
    "src/direct_message_crypto/tests.rs",
];
const REQUIRED_ROOT_MARKERS: &[&str] = &[
    "mod models;",
    "mod errors;",
    "mod engine;",
    "mod key_agreement;",
    "mod cipher;",
    "mod validation;",
    "mod encoding;",
    "#[cfg(test)]",
    "mod tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub struct DirectMessageCiphertext {",
    "pub struct DirectMessageCryptoEngine {",
    "pub enum DirectMessageCryptoError {",
    "fn derive_x25519_shared_secret(",
    "fn decrypt_with_compatibility_candidates(",
    "fn validate_ciphertext_context(",
    "fn hex_decode(",
    "mod tests {",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn assert_required_files_exist() {
    for path in REQUIRED_FILES {
        let file = read_repo_file(path);
        assert!(
            !file.trim().is_empty(),
            "expected extracted module file {path} to exist and be non-empty"
        );
    }
}

fn assert_root_markers(root: &str) {
    for marker in REQUIRED_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "expected root shell to contain marker `{marker}`"
        );
    }
}

fn assert_moved_markers_absent(root: &str) {
    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "expected root shell to move marker `{marker}` into extracted modules"
        );
    }
}

#[test]
fn direct_message_crypto_root_is_extracted() {
    let root = read_repo_file(ROOT);
    let line_count = root.lines().count();
    assert!(
        line_count <= MAX_ROOT_LINES,
        "{ROOT} should be <= {MAX_ROOT_LINES} lines after extraction, found {line_count}"
    );
    assert_required_files_exist();
    assert_root_markers(&root);
    assert_moved_markers_absent(&root);
}
