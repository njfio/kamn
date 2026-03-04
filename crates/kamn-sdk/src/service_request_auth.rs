use crate::{AgentDid, SdkError};

fn validate_header_field(field: &'static str, value: &str) -> Result<(), SdkError> {
    super::service_http_io::validate_http_header_value(field, value)
}

/// Request authentication envelope for service API routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequestAuth {
    sender_did: AgentDid,
    nonce: u64,
    signature: String,
    signer_public_key_hex: Option<String>,
    scope: Option<String>,
}

impl ServiceRequestAuth {
    /// Builds a validated request auth envelope.
    pub fn new(sender_did: AgentDid, nonce: u64, signature: String) -> Result<Self, SdkError> {
        Self::new_with_signer_public_key_and_scope(sender_did, nonce, signature, None, None)
    }

    /// Builds a validated request auth envelope with optional auth scope marker.
    pub fn new_with_scope(
        sender_did: AgentDid,
        nonce: u64,
        signature: String,
        scope: Option<&str>,
    ) -> Result<Self, SdkError> {
        Self::new_with_signer_public_key_and_scope(sender_did, nonce, signature, None, scope)
    }

    /// Builds a validated request auth envelope with optional signer key and auth scope marker.
    pub fn new_with_signer_public_key_and_scope(
        sender_did: AgentDid,
        nonce: u64,
        signature: String,
        signer_public_key_hex: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Self, SdkError> {
        if nonce == 0 {
            return Err(SdkError::InvalidInput {
                field: "request_auth.nonce",
                reason: "must be greater than zero",
            });
        }
        let normalized_signature = signature.trim();
        if normalized_signature.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "request_auth.signature",
                reason: "must not be empty",
            });
        }
        validate_header_field("request_auth.sender_did", sender_did.as_str())?;
        validate_header_field("request_auth.signature", normalized_signature)?;
        let scope = match scope {
            Some(scope) => {
                let normalized = scope.trim();
                if normalized.is_empty() {
                    return Err(SdkError::InvalidInput {
                        field: "request_auth.scope",
                        reason: "must not be empty when set",
                    });
                }
                validate_header_field("request_auth.scope", normalized)?;
                Some(normalized.to_owned())
            }
            None => None,
        };
        let signer_public_key_hex = match signer_public_key_hex {
            Some(value) => {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return Err(SdkError::InvalidInput {
                        field: "request_auth.signer_public_key",
                        reason: "must not be empty when set",
                    });
                }
                validate_header_field("request_auth.signer_public_key", normalized)?;
                Some(normalized.to_owned())
            }
            None => None,
        };
        Ok(Self {
            sender_did,
            nonce,
            signature: normalized_signature.to_owned(),
            signer_public_key_hex,
            scope,
        })
    }

    pub(super) fn sender_did(&self) -> &AgentDid {
        &self.sender_did
    }

    pub(super) fn nonce(&self) -> u64 {
        self.nonce
    }

    pub(super) fn signature(&self) -> &str {
        self.signature.as_str()
    }

    pub(super) fn signer_public_key_hex(&self) -> Option<&str> {
        self.signer_public_key_hex.as_deref()
    }

    pub(super) fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }
}
