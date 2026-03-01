use kamn_data_layer::data_layer_hashing::{validated_tagged_sha256, DataLayerHashingError};

#[test]
fn integration_validated_tagged_sha256_accepts_canonical_label() {
    let tagged = validated_tagged_sha256("abc", "sha256").expect("canonical label should pass");
    assert_eq!(
        tagged,
        "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}

#[test]
fn integration_validated_tagged_sha256_rejects_empty_label() {
    assert_eq!(
        validated_tagged_sha256("abc", ""),
        Err(DataLayerHashingError::EmptyAlgorithmLabel)
    );
}

#[test]
fn integration_validated_tagged_sha256_rejects_malformed_label() {
    assert_eq!(
        validated_tagged_sha256("abc", "SHA_256"),
        Err(DataLayerHashingError::InvalidAlgorithmLabel)
    );
    assert_eq!(
        validated_tagged_sha256("abc", "sha256 "),
        Err(DataLayerHashingError::InvalidAlgorithmLabel)
    );
}
