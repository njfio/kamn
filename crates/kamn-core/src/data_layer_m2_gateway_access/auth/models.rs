use crate::data_layer_m2_gateway_access::models::{parse_agent_did, DataLayerM2GatewayError};

/// Input for DID-authenticated session issuance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidAuthRequest {
    /// Requester DID.
    pub requester_did: String,
    /// Challenge/nonce bound to credential signature.
    pub challenge: String,
    /// Credential payload carrying deterministic signature binding.
    pub credential: String,
    /// Request issuance timestamp in epoch seconds.
    pub issued_at_epoch_seconds: u64,
    /// Requested session TTL in seconds.
    pub ttl_seconds: u64,
}

/// Typed validated auth request used by internal M2 session issuance paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidAuthRequestValidated {
    /// Canonical requester agent DID.
    pub requester_did: crate::AgentDid,
    /// Challenge/nonce bound to credential signature.
    pub challenge: String,
    /// Credential payload carrying deterministic signature binding.
    pub credential: String,
    /// Request issuance timestamp in epoch seconds.
    pub issued_at_epoch_seconds: u64,
    /// Requested session TTL in seconds.
    pub ttl_seconds: u64,
}

impl TryFrom<DataLayerM2DidAuthRequest> for DataLayerM2DidAuthRequestValidated {
    type Error = DataLayerM2GatewayError;

    fn try_from(request: DataLayerM2DidAuthRequest) -> Result<Self, Self::Error> {
        validate_non_empty(request.challenge.as_str(), "challenge")?;
        validate_non_empty(request.credential.as_str(), "credential")?;
        Ok(Self {
            requester_did: parse_agent_did(
                request.requester_did.as_str(),
                "requester_did",
                super::super::DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
            )?,
            challenge: request.challenge,
            credential: request.credential,
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
            ttl_seconds: request.ttl_seconds,
        })
    }
}

fn validate_non_empty(value: &str, field: &'static str) -> Result<(), DataLayerM2GatewayError> {
    if value.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField(field));
    }
    Ok(())
}

/// Session token issued by M2 DID authentication.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2SessionToken {
    /// Stable session token identifier.
    pub token_id: String,
    /// Authenticated requester DID.
    pub requester_did: String,
    /// Session issuance timestamp.
    pub issued_at_epoch_seconds: u64,
    /// Session expiry timestamp.
    pub expires_at_epoch_seconds: u64,
}
