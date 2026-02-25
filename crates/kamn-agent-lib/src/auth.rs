use crate::errors::AgentLibError;
use kamn_sdk::{service_signature_for_state_hash_with_private_key, AgentDid, SdkError};

/// Cryptographic auth header bundle for Service API requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KamnAuthHeaders {
    /// `x-kamn-sender-did` header value.
    pub sender_did_header: String,
    /// `x-kamn-request-nonce` header value.
    pub nonce_header: String,
    /// `x-kamn-request-signature` header value.
    pub signature_header: String,
    /// Optional `x-kamn-authz-scope` header value.
    pub authz_scope_header: Option<String>,
}

impl KamnAuthHeaders {
    /// Builds cryptographic auth headers from canonical signature fields.
    pub fn build(
        sender_did: &str,
        signing_key: &str,
        nonce: u64,
        state_hash: &str,
        body: &[u8],
        authz_scope: Option<&str>,
    ) -> Result<Self, AgentLibError> {
        if signing_key.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "signing_key",
                reason: "must not be empty".to_owned(),
            });
        }
        if nonce == 0 {
            return Err(AgentLibError::InvalidInput {
                field: "nonce",
                reason: "must be greater than zero".to_owned(),
            });
        }
        if state_hash.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "state_hash",
                reason: "must not be empty".to_owned(),
            });
        }

        let sender_did = AgentDid::parse(sender_did.to_owned())?;
        let body_str = std::str::from_utf8(body).map_err(|_| AgentLibError::InvalidInput {
            field: "body",
            reason: "must be utf-8".to_owned(),
        })?;
        let signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            nonce,
            state_hash,
            body_str,
            signing_key,
        )
        .map_err(|error| match error {
            SdkError::InvalidInput {
                field: "service.request_auth.private_key",
                ..
            } => AgentLibError::InvalidInput {
                field: "signing_key",
                reason: "must be valid secp256k1 private key hex".to_owned(),
            },
            _ => AgentLibError::Sdk(error),
        })?;

        Ok(Self {
            sender_did_header: sender_did.to_string(),
            nonce_header: nonce.to_string(),
            signature_header: signature,
            authz_scope_header: authz_scope.map(str::to_owned),
        })
    }
}
