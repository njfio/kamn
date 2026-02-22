use crate::errors::AgentLibError;
use crate::identity::AgentIdentity;
use kamn_sdk::{signature_for_fields, AgentDid};

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
    /// Deterministic signature marker.
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
        let expected = signature_for_fields(
            self.from.as_str(),
            self.nonce,
            self.state_hash.as_str(),
            self.body.as_str(),
        );
        if self.signature != expected {
            return Err(AgentLibError::InvalidInput {
                field: "signature",
                reason: "does not match canonical envelope fields".to_owned(),
            });
        }
        Ok(())
    }
}

/// Builds a canonical message envelope and deterministic signature.
pub fn build_and_sign_envelope(
    identity: &AgentIdentity,
    to: &str,
    state_hash: &str,
    nonce: u64,
    body: &str,
) -> Result<CanonicalMessageEnvelope, AgentLibError> {
    let to_did = AgentDid::parse(to.to_owned())?;
    let envelope = CanonicalMessageEnvelope {
        from: identity.did().to_string(),
        to: to_did.to_string(),
        nonce,
        state_hash: state_hash.to_owned(),
        body: body.to_owned(),
        signature: signature_for_fields(identity.did().as_str(), nonce, state_hash, body),
    };
    envelope.verify_integrity()?;
    Ok(envelope)
}
