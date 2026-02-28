use crate::errors::AgentLibError;
use crate::identity::AgentIdentity;
use kamn_sdk::{
    service_public_key_for_private_key, service_signature_for_state_hash_with_private_key,
    service_verify_signature_with_public_key, AgentDid, SdkError,
};

/// Canonical message envelope emitted by phase-1 agent operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalMessageEnvelope {
    /// Sender DID.
    pub from: String,
    /// Recipient DID.
    pub to: String,
    /// Monotonic nonce.
    pub nonce: u64,
    /// Runtime state hash marker.
    pub state_hash: String,
    /// Message body.
    pub body: String,
    /// Compressed secp256k1 signer public key hex.
    pub signer_public_key: String,
    /// Cryptographic signature marker.
    pub signature: String,
}

impl CanonicalMessageEnvelope {
    /// Verifies deterministic envelope integrity checks.
    pub fn verify_integrity(&self) -> Result<(), AgentLibError> {
        AgentDid::parse(self.from.clone())?;
        AgentDid::parse(self.to.clone())?;
        if self.nonce == 0 {
            return Err(AgentLibError::InvalidInput {
                field: "nonce",
                reason: "must be greater than zero".to_owned(),
            });
        }
        if self.state_hash.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "state_hash",
                reason: "must not be empty".to_owned(),
            });
        }
        if self.body.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "body",
                reason: "must not be empty".to_owned(),
            });
        }
        if self.signer_public_key.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "signer_public_key",
                reason: "must not be empty".to_owned(),
            });
        }
        let from_did = AgentDid::parse(self.from.clone())?;
        service_verify_signature_with_public_key(
            &from_did,
            self.nonce,
            self.state_hash.as_str(),
            self.body.as_str(),
            self.signature.as_str(),
            self.signer_public_key.as_str(),
        )
        .map_err(map_sdk_signature_error)?;
        Ok(())
    }
}

fn map_sdk_signature_error(error: SdkError) -> AgentLibError {
    match error {
        SdkError::InvalidInput {
            field: "service.request_auth.expected_public_key",
            ..
        } => AgentLibError::InvalidInput {
            field: "signer_public_key",
            reason: "must be valid compressed secp256k1 public key hex".to_owned(),
        },
        _ => AgentLibError::InvalidInput {
            field: "signature",
            reason: "does not match canonical envelope fields".to_owned(),
        },
    }
}

/// Builds a canonical message envelope and cryptographic signature.
pub fn build_and_sign_envelope(
    identity: &AgentIdentity,
    to: &str,
    state_hash: &str,
    nonce: u64,
    body: &str,
) -> Result<CanonicalMessageEnvelope, AgentLibError> {
    let to_did = AgentDid::parse(to)?;
    let signature = service_signature_for_state_hash_with_private_key(
        identity.did(),
        nonce,
        state_hash,
        body,
        identity.signing_key(),
    )
    .map_err(|error| match error {
        SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            ..
        } => AgentLibError::InvalidInput {
            field: "signing_key",
            reason: "must be valid secp256k1 private key hex".to_owned(),
        },
        other => map_sdk_signature_error(other),
    })?;
    let signer_public_key = service_public_key_for_private_key(identity.signing_key()).map_err(
        |error| match error {
            SdkError::InvalidInput {
                field: "service.request_auth.private_key",
                ..
            } => AgentLibError::InvalidInput {
                field: "signing_key",
                reason: "must be valid secp256k1 private key hex".to_owned(),
            },
            other => map_sdk_signature_error(other),
        },
    )?;
    let envelope = CanonicalMessageEnvelope {
        from: identity.did().to_string(),
        to: to_did.to_string(),
        nonce,
        state_hash: state_hash.to_owned(),
        body: body.to_owned(),
        signer_public_key,
        signature,
    };
    envelope.verify_integrity()?;
    Ok(envelope)
}
