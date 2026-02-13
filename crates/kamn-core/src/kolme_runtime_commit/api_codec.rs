//! Kolme API request/response codec ownership.

use super::{
    KamnKolmeApiBroadcastRequest, KamnKolmeApiBroadcastResponse, KamnKolmeApiCodecError,
    KamnKolmeApiNextNonceRequest, KamnKolmeApiNextNonceResponse, KolmeRuntimeCommitError,
};
use kamn_kolme::KolmeRuntimeCommitProviderError;

/// Typed nonce lookup request for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceRequest {
    /// Public key used to resolve next nonce and account identity.
    pub pubkey: String,
}

impl KolmeApiNextNonceRequest {
    /// Builds a deterministic nonce lookup request.
    pub fn new(pubkey: &str) -> Result<Self, KolmeRuntimeCommitError> {
        let extracted = KamnKolmeApiNextNonceRequest::new(pubkey).map_err(|error| match error {
            KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                KolmeRuntimeCommitError::InvalidRequest { field, reason }
            }
            KamnKolmeApiCodecError::MalformedResponse { .. } => {
                KolmeRuntimeCommitError::InvalidRequest {
                    field: "codec_payload",
                    reason: "must be valid json",
                }
            }
        })?;
        Ok(Self {
            pubkey: extracted.pubkey,
        })
    }

    /// Returns encoded request path for the configured nonce endpoint.
    pub fn query_path(&self, nonce_path: &str) -> String {
        KamnKolmeApiNextNonceRequest {
            pubkey: self.pubkey.clone(),
        }
        .query_path(nonce_path)
    }
}

/// Typed nonce lookup response for Kolme `/get-next-nonce`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiNextNonceResponse {
    /// Monotonic next nonce for the provided public key.
    pub next_nonce: u64,
    /// Optional account identifier mapped to the provided public key.
    pub account_id: Option<String>,
}

impl KolmeApiNextNonceResponse {
    /// Parses one nonce lookup response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeRuntimeCommitProviderError> {
        let extracted =
            KamnKolmeApiNextNonceResponse::parse_json(response).map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse {
                        reason: format!("invalid request {field}: {reason}"),
                    }
                }
                KamnKolmeApiCodecError::MalformedResponse { reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                }
            })?;
        Ok(Self {
            next_nonce: extracted.next_nonce,
            account_id: extracted.account_id,
        })
    }
}

/// Typed broadcast request payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastRequest {
    /// Tagged transaction message payload.
    pub message: String,
    /// Chain signature for the transaction message payload.
    pub signature: String,
    /// Signature recovery identifier.
    pub recovery_id: u8,
}

impl KolmeApiBroadcastRequest {
    /// Builds a deterministic broadcast request payload.
    pub fn new(
        message: &str,
        signature: &str,
        recovery_id: u8,
    ) -> Result<Self, KolmeRuntimeCommitError> {
        let extracted = KamnKolmeApiBroadcastRequest::new(message, signature, recovery_id)
            .map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitError::InvalidRequest { field, reason }
                }
                KamnKolmeApiCodecError::MalformedResponse { .. } => {
                    KolmeRuntimeCommitError::InvalidRequest {
                        field: "codec_payload",
                        reason: "must be valid json",
                    }
                }
            })?;
        Ok(Self {
            message: extracted.message,
            signature: extracted.signature,
            recovery_id: extracted.recovery_id,
        })
    }

    /// Returns deterministic JSON payload in canonical field order.
    pub fn to_json_payload(&self) -> String {
        KamnKolmeApiBroadcastRequest {
            message: self.message.clone(),
            signature: self.signature.clone(),
            recovery_id: self.recovery_id,
        }
        .to_json_payload()
    }
}

/// Typed broadcast response payload for Kolme `/broadcast`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeApiBroadcastResponse {
    /// Transaction hash identifier from broadcast response.
    pub txhash: String,
}

impl KolmeApiBroadcastResponse {
    /// Parses one broadcast response JSON payload.
    pub fn parse_json(response: &str) -> Result<Self, KolmeRuntimeCommitProviderError> {
        let extracted =
            KamnKolmeApiBroadcastResponse::parse_json(response).map_err(|error| match error {
                KamnKolmeApiCodecError::InvalidRequest { field, reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse {
                        reason: format!("invalid request {field}: {reason}"),
                    }
                }
                KamnKolmeApiCodecError::MalformedResponse { reason } => {
                    KolmeRuntimeCommitProviderError::MalformedResponse { reason }
                }
            })?;
        Ok(Self {
            txhash: extracted.txhash,
        })
    }
}
