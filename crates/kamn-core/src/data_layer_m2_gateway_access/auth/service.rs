use super::models::{
    DataLayerM2DidAuthRequest, DataLayerM2DidAuthRequestValidated, DataLayerM2SessionToken,
};
use crate::data_layer_m2_gateway_access::models::{tagged_digest, DataLayerM2GatewayError};

/// Deterministic DID session service for M2 gateway contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2DidSessionService {
    max_ttl_seconds: u64,
}

impl DataLayerM2DidSessionService {
    /// Constructs a DID session service with max allowed TTL.
    pub fn new(max_ttl_seconds: u64) -> Result<Self, DataLayerM2GatewayError> {
        if max_ttl_seconds == 0 {
            return Err(DataLayerM2GatewayError::InvalidSessionTtl {
                ttl_seconds: 0,
                max_ttl_seconds,
            });
        }
        Ok(Self { max_ttl_seconds })
    }

    /// Authenticates a DID-bound request and issues a deterministic session token.
    pub fn authenticate(
        &self,
        request: DataLayerM2DidAuthRequest,
    ) -> Result<DataLayerM2SessionToken, DataLayerM2GatewayError> {
        let request = DataLayerM2DidAuthRequestValidated::try_from(request)?;
        self.validate_ttl(request.ttl_seconds)?;
        self.validate_credential(&request)?;
        self.issue_token(request)
    }

    fn validate_ttl(&self, ttl_seconds: u64) -> Result<(), DataLayerM2GatewayError> {
        if ttl_seconds == 0 || ttl_seconds > self.max_ttl_seconds {
            return Err(DataLayerM2GatewayError::InvalidSessionTtl {
                ttl_seconds,
                max_ttl_seconds: self.max_ttl_seconds,
            });
        }
        Ok(())
    }

    fn validate_credential(
        &self,
        request: &DataLayerM2DidAuthRequestValidated,
    ) -> Result<(), DataLayerM2GatewayError> {
        let requester_did = request.requester_did.as_str();
        let expected = format!("sig:{requester_did}:{}", request.challenge);
        if crate::constant_time_eq::constant_time_eq_bytes(
            request.credential.as_bytes(),
            expected.as_bytes(),
        ) {
            return Ok(());
        }
        Err(DataLayerM2GatewayError::InvalidCredential(
            "credential signature mismatch".to_owned(),
        ))
    }

    fn issue_token(
        &self,
        request: DataLayerM2DidAuthRequestValidated,
    ) -> Result<DataLayerM2SessionToken, DataLayerM2GatewayError> {
        let requester_did = request.requester_did.as_str().to_owned();
        let expires_at_epoch_seconds = request
            .issued_at_epoch_seconds
            .checked_add(request.ttl_seconds)
            .ok_or(DataLayerM2GatewayError::SessionExpiryOverflow)?;
        Ok(DataLayerM2SessionToken {
            token_id: session_token_id(
                &requester_did,
                &request.challenge,
                request.issued_at_epoch_seconds,
                expires_at_epoch_seconds,
            ),
            requester_did,
            issued_at_epoch_seconds: request.issued_at_epoch_seconds,
            expires_at_epoch_seconds,
        })
    }
}

fn session_token_id(
    requester_did: &str,
    challenge: &str,
    issued_at_epoch_seconds: u64,
    expires_at_epoch_seconds: u64,
) -> String {
    format!(
        "session:{}",
        tagged_digest(
            format!(
                "did-session|did:{requester_did}|challenge:{challenge}|issued:{issued_at_epoch_seconds}|expires:{expires_at_epoch_seconds}"
            )
            .as_str()
        )
    )
}
