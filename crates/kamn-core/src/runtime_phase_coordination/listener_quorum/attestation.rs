use super::super::did_validation::parse_listener_did;
use super::ListenerQuorumError;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Listener attestation.
pub struct ListenerAttestation {
    listener_did: String,
    attestation_id: String,
}

impl ListenerAttestation {
    /// Handles new.
    pub fn new(listener_did: &str, attestation_id: &str) -> Result<Self, ListenerQuorumError> {
        parse_listener_did(listener_did, "listener_did")?;
        if attestation_id.trim().is_empty() {
            return Err(ListenerQuorumError::InvalidAttestationId);
        }
        Ok(Self {
            listener_did: listener_did.to_owned(),
            attestation_id: attestation_id.to_owned(),
        })
    }

    /// Handles listener did.
    pub fn listener_did(&self) -> &str {
        &self.listener_did
    }

    /// Handles attestation id.
    pub fn attestation_id(&self) -> &str {
        &self.attestation_id
    }
}
