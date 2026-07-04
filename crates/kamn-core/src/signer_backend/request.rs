use super::env::resolve_signer_private_key_hex;
use super::errors::SignerBackendError;
use super::signing_cache::SignerBackendSigningCache;
use crate::signature_profile::service_auth_sign_with_private_key_hex;
use crate::transaction::BaselineTransaction;

/// Canonical payload sent to signer backends for signature generation and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningRequest {
    /// Key reference used to route signing to local or secure providers.
    pub key_id: String,
    /// Logical sender identity used in signature preimage construction.
    pub sender: String,
    /// Monotonic nonce bound to the signature preimage.
    pub nonce: u64,
    /// Serialized payload bytes represented as UTF-8 string.
    pub payload: String,
    /// State-hash binding used to anchor signature replay protection.
    pub state_hash: String,
}

impl SigningRequest {
    /// Build and validate a signing request from raw request components.
    pub fn new(
        key_id: &str,
        sender: &str,
        nonce: u64,
        payload: &str,
        state_hash: &str,
    ) -> Result<Self, SignerBackendError> {
        validate_request_fields(key_id, sender, nonce, payload, state_hash)?;
        Ok(build_request(key_id, sender, nonce, payload, state_hash))
    }

    /// Build a signing request from a canonical baseline transaction record.
    pub fn for_transaction(
        key_id: &str,
        tx: &BaselineTransaction,
    ) -> Result<Self, SignerBackendError> {
        if tx.id.trim().is_empty() {
            return Err(SignerBackendError::EmptyField("transaction_id"));
        }
        Self::new(key_id, &tx.sender, tx.nonce, &tx.payload, &tx.state_hash)
    }

    pub(super) fn expected_signature(&self) -> Result<String, SignerBackendError> {
        let private_key_hex = resolve_signer_private_key_hex(&self.key_id)?;
        service_auth_sign_with_private_key_hex(
            &self.sender,
            self.nonce,
            &self.state_hash,
            &self.payload,
            private_key_hex.as_str(),
        )
        .map_err(|_| SignerBackendError::InvalidSigningKeyMaterial {
            key_id: self.key_id.clone(),
        })
    }

    pub(crate) fn expected_signature_with_cache(
        &self,
        signing_cache: &SignerBackendSigningCache,
    ) -> Result<String, SignerBackendError> {
        signing_cache.expected_signature(self)
    }
}

fn validate_request_fields(
    key_id: &str,
    sender: &str,
    nonce: u64,
    payload: &str,
    state_hash: &str,
) -> Result<(), SignerBackendError> {
    validate_non_empty("key_id", key_id)?;
    validate_non_empty("sender", sender)?;
    validate_nonce(nonce)?;
    validate_non_empty("payload", payload)?;
    validate_non_empty("state_hash", state_hash)
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), SignerBackendError> {
    if value.trim().is_empty() {
        return Err(SignerBackendError::EmptyField(field));
    }
    Ok(())
}

fn validate_nonce(nonce: u64) -> Result<(), SignerBackendError> {
    if nonce == 0 {
        return Err(SignerBackendError::InvalidNonce);
    }
    Ok(())
}

fn build_request(
    key_id: &str,
    sender: &str,
    nonce: u64,
    payload: &str,
    state_hash: &str,
) -> SigningRequest {
    SigningRequest {
        key_id: key_id.to_owned(),
        sender: sender.to_owned(),
        nonce,
        payload: payload.to_owned(),
        state_hash: state_hash.to_owned(),
    }
}
