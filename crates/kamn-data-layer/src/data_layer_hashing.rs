use k256::sha2::{Digest, Sha256};

/// Typed validation errors for data-layer hashing helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerHashingError {
    /// Algorithm label was empty or whitespace-only.
    EmptyAlgorithmLabel,
    /// Algorithm label contains disallowed characters.
    InvalidAlgorithmLabel,
}

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

/// Prefixes SHA-256 digest output with a validated canonical algorithm label.
pub fn validated_tagged_sha256(
    value: &str,
    algorithm_label: &str,
) -> Result<String, DataLayerHashingError> {
    validate_algorithm_label(algorithm_label)?;
    Ok(tagged_sha256(value, algorithm_label))
}

fn validate_algorithm_label(algorithm_label: &str) -> Result<(), DataLayerHashingError> {
    let label = algorithm_label.trim();
    if label.is_empty() {
        return Err(DataLayerHashingError::EmptyAlgorithmLabel);
    }
    if !label
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
    {
        return Err(DataLayerHashingError::InvalidAlgorithmLabel);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{sha256_hex, tagged_sha256, validated_tagged_sha256, DataLayerHashingError};

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

    #[test]
    fn tagged_sha256_prefixes_algorithm_label_and_digest() {
        let tagged = tagged_sha256("abc", "sha256");
        assert_eq!(
            tagged,
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn validated_tagged_sha256_rejects_noncanonical_labels() {
        assert_eq!(
            validated_tagged_sha256("abc", " "),
            Err(DataLayerHashingError::EmptyAlgorithmLabel)
        );
        assert_eq!(
            validated_tagged_sha256("abc", "SHA_256"),
            Err(DataLayerHashingError::InvalidAlgorithmLabel)
        );
    }
}
