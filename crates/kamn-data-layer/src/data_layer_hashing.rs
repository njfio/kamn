use k256::sha2::{Digest, Sha256};

/// Computes lowercase hexadecimal SHA-256 over UTF-8 payload bytes.
pub fn sha256_hex(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(value.as_bytes());
    let mut output = String::with_capacity(64);
    for byte in digest {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

/// Prefixes SHA-256 digest output with the provided algorithm label.
pub fn tagged_sha256(value: &str, algorithm_label: &str) -> String {
    format!("{algorithm_label}:{}", sha256_hex(value))
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn regression_issue_5922_sha256_hex_matches_known_test_vectors() {
        // Regression: #5922
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
