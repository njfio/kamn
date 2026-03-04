use crate::{AgentDid, SdkError};
use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    service_auth_verify_with_public_key_hex, ServiceAuthSignatureError,
    SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
};

fn service_private_key_hex_from_env() -> Result<String, SdkError> {
    std::env::var(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV).map_err(|_| SdkError::InvalidInput {
        field: "service.request_auth.private_key",
        reason: "missing KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
    })
}

/// Deterministic request signature builder for service API fields.
pub fn service_signature_for_fields(
    sender_did: &AgentDid,
    nonce: u64,
    chain_id: &str,
    chain_version: &str,
    body: &str,
) -> Result<String, SdkError> {
    let private_key_hex = service_private_key_hex_from_env()?;
    let state_hash = format!("service-api:{chain_id}:{chain_version}");
    service_signature_for_state_hash_with_private_key(
        sender_did,
        nonce,
        state_hash.as_str(),
        body,
        private_key_hex.as_str(),
    )
}

/// Derives signer public key hex from the configured service signing private key env.
pub fn service_signer_public_key_for_fields() -> Result<String, SdkError> {
    let private_key_hex = service_private_key_hex_from_env()?;
    service_public_key_for_private_key(private_key_hex.as_str())
}

/// Cryptographic request signature builder for canonical service state-hash fields.
pub fn service_signature_for_state_hash_with_private_key(
    sender_did: &AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
    private_key_hex: &str,
) -> Result<String, SdkError> {
    service_auth_sign_with_private_key_hex(
        sender_did.as_str(),
        nonce,
        state_hash,
        body,
        private_key_hex,
    )
    .map_err(map_service_auth_error_to_sdk)
}

/// Derives compressed secp256k1 public key hex from private key material.
pub fn service_public_key_for_private_key(private_key_hex: &str) -> Result<String, SdkError> {
    service_auth_public_key_hex_from_private_key_hex(private_key_hex).map_err(|error| match error {
        ServiceAuthSignatureError::EmptyField("private_key_hex")
        | ServiceAuthSignatureError::InvalidPrivateKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "must be valid secp256k1 private key hex",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "failed to derive secp256k1 signer public key",
        },
    })
}

/// Verifies a service signature against canonical state-hash fields and signer public key.
pub fn service_verify_signature_with_public_key(
    sender_did: &AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
    signature: &str,
    signer_public_key_hex: &str,
) -> Result<(), SdkError> {
    service_auth_verify_with_public_key_hex(
        signature,
        sender_did.as_str(),
        nonce,
        state_hash,
        body,
        signer_public_key_hex,
    )
    .map_err(|error| match error {
        ServiceAuthSignatureError::EmptyField("expected_public_key_hex")
        | ServiceAuthSignatureError::InvalidPublicKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.expected_public_key",
            reason: "must be valid compressed secp256k1 public key hex",
        },
        ServiceAuthSignatureError::EmptyField("state_hash") => SdkError::InvalidInput {
            field: "service.request_auth.state_hash",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::EmptyField("signature") => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::InvalidNonce => SdkError::InvalidInput {
            field: "service.request_auth.nonce",
            reason: "must be greater than zero",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "failed cryptographic signature verification",
        },
    })
}

fn map_service_auth_error_to_sdk(error: ServiceAuthSignatureError) -> SdkError {
    match error {
        ServiceAuthSignatureError::EmptyField("private_key_hex")
        | ServiceAuthSignatureError::InvalidPrivateKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "must be valid secp256k1 private key hex",
        },
        ServiceAuthSignatureError::EmptyField("state_hash") => SdkError::InvalidInput {
            field: "service.request_auth.state_hash",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::InvalidNonce => SdkError::InvalidInput {
            field: "service.request_auth.nonce",
            reason: "must be greater than zero",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "failed to produce cryptographic service signature",
        },
    }
}
