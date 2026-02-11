//! Provider finality receipt parsing contracts for runtime-commit status responses.

use crate::{
    parse_commit_id_from_response_fields, parse_commit_receipt_finality,
    parse_provider_response_fields, required_provider_response_field, KolmeCommitReceiptFinality,
    KolmeProviderOutcomePolicyError, KolmeProviderResponsePolicyError, ReceiptFinalityError,
};
use std::error::Error;
use std::fmt;

/// Deterministic provider receipt parsed from one finality response payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeProviderFinalityReceipt {
    /// Provider identifier.
    pub provider: String,
    /// Observed deterministic commit identifier.
    pub commit_id: String,
    /// Parsed commit finality.
    pub finality: KolmeCommitReceiptFinality,
}

/// Error returned when parsing provider finality receipt payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeProviderFinalityReceiptPolicyError {
    /// Payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeProviderFinalityReceiptPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeProviderFinalityReceiptPolicyError {}

/// Parses one provider finality response and validates commit-id correlation.
pub fn parse_provider_finality_receipt(
    response: &str,
    expected_commit_id: &str,
) -> Result<KolmeProviderFinalityReceipt, KolmeProviderFinalityReceiptPolicyError> {
    let expected_commit_id = expected_commit_id.trim();
    if expected_commit_id.is_empty() {
        return Err(KolmeProviderFinalityReceiptPolicyError::MalformedResponse {
            reason: "commit_id must not be empty".to_owned(),
        });
    }

    let fields = parse_provider_response_fields(response).map_err(map_provider_response_error)?;
    let provider = required_provider_response_field(&fields, "provider")
        .map_err(map_provider_outcome_error)?;
    let observed_commit_id =
        parse_commit_id_from_response_fields(&fields).map_err(map_provider_outcome_error)?;
    if observed_commit_id != expected_commit_id {
        return Err(KolmeProviderFinalityReceiptPolicyError::MalformedResponse {
            reason: format!(
                "commit_id mismatch: expected '{expected_commit_id}', observed '{observed_commit_id}'"
            ),
        });
    }

    let finality_value = required_provider_response_field(&fields, "finality")
        .map_err(map_provider_outcome_error)?;
    let finality =
        parse_commit_receipt_finality(finality_value.as_str()).map_err(map_finality_error)?;

    Ok(KolmeProviderFinalityReceipt {
        provider,
        commit_id: observed_commit_id,
        finality,
    })
}

fn map_provider_response_error(
    error: KolmeProviderResponsePolicyError,
) -> KolmeProviderFinalityReceiptPolicyError {
    match error {
        KolmeProviderResponsePolicyError::MalformedResponse { reason } => {
            KolmeProviderFinalityReceiptPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_provider_outcome_error(
    error: KolmeProviderOutcomePolicyError,
) -> KolmeProviderFinalityReceiptPolicyError {
    match error {
        KolmeProviderOutcomePolicyError::MalformedResponse { reason } => {
            KolmeProviderFinalityReceiptPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_finality_error(error: ReceiptFinalityError) -> KolmeProviderFinalityReceiptPolicyError {
    KolmeProviderFinalityReceiptPolicyError::MalformedResponse {
        reason: error.to_string(),
    }
}
