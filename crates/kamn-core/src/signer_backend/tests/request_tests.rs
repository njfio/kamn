use crate::signer_backend::provider_policy::{
    CanonicalSecureKeyReference, SecureSignerProvider, SignerKeyRole,
};
use crate::signer_backend::request::SigningRequest;
use crate::signer_backend::SignerBackendError;

#[test]
fn signing_request_rejects_invalid_fields() {
    assert_eq!(
        SigningRequest::new("", "agent-a", 1, "payload", "state:genesis"),
        Err(SignerBackendError::EmptyField("key_id"))
    );
    assert_eq!(
        SigningRequest::new("secure:key-1", "agent-a", 0, "payload", "state:genesis"),
        Err(SignerBackendError::InvalidNonce)
    );
}

#[test]
fn secure_provider_parser_accepts_legacy_and_explicit_key_formats() {
    assert_eq!(
        SecureSignerProvider::from_key_id("secure:key-legacy-1"),
        Ok(SecureSignerProvider::Mock)
    );
    assert_eq!(
        SecureSignerProvider::from_key_id("secure:mock:key-legacy-2"),
        Ok(SecureSignerProvider::Mock)
    );
    assert_eq!(
        SecureSignerProvider::from_key_id("secure:aws-kms:key-prod-1"),
        Ok(SecureSignerProvider::AwsKmsEmulator)
    );
}

#[test]
fn secure_provider_parser_rejects_unknown_and_malformed_key_references() {
    assert_eq!(
        SecureSignerProvider::from_key_id("secure:gcp-kms:key-prod-1"),
        Err(SignerBackendError::UnsupportedSecureProvider {
            backend: "secure-mock".to_owned(),
            provider: "gcp-kms".to_owned(),
            key_id: "secure:gcp-kms:key-prod-1".to_owned(),
        })
    );
    assert_eq!(
        SecureSignerProvider::from_key_id("secure:"),
        Err(SignerBackendError::MalformedSecureKeyReference {
            key_id: "secure:".to_owned(),
        })
    );
}

#[test]
fn signer_key_role_parser_supports_role_prefixes_and_legacy_defaults() {
    assert_eq!(
        SignerKeyRole::from_key_id("secure:key-legacy-1"),
        Ok(SignerKeyRole::Operator)
    );
    assert_eq!(
        SignerKeyRole::from_key_id("secure:aws-kms:role-admin/key-prod-1"),
        Ok(SignerKeyRole::Admin)
    );
    assert_eq!(
        SignerKeyRole::from_key_id("secure:aws-kms:role-treasury/key-prod-2"),
        Ok(SignerKeyRole::Treasury)
    );
}

#[test]
fn signer_key_role_parser_rejects_unsupported_role_labels() {
    assert_eq!(
        SignerKeyRole::from_key_id("secure:aws-kms:role-root/key-prod-3"),
        Err(SignerBackendError::UnsupportedSignerKeyRole {
            role: "root".to_owned(),
            key_id: "secure:aws-kms:role-root/key-prod-3".to_owned(),
        })
    );
}

#[test]
fn canonical_secure_key_reference_parser_preserves_provider_key_scope() {
    let parsed = CanonicalSecureKeyReference::parse("secure:AWS-KMS:role-TREASURY/key-prod-9")
        .expect("canonical parser should accept provider + role scoped keys");
    assert_eq!(parsed.provider, SecureSignerProvider::AwsKmsEmulator);
    assert_eq!(parsed.key_role, SignerKeyRole::Treasury);
    assert_eq!(parsed.provider_key_id, "role-TREASURY/key-prod-9");
}
