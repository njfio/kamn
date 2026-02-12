//! Provider outcome parsing and commit-id helper contracts for runtime commits.

use crate::{
    parse_provider_response_fields, parse_receipt_finality,
    runtime_lifecycle_policy::{commit_finality_from_receipt_finality, KolmeCommitReceiptFinality},
    KolmeProviderResponsePolicyError, ReceiptFinality, ReceiptFinalityError,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Typed provider outcome extracted from one live response payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeProviderOutcome {
    /// Provider accepted a new request submission.
    Submitted {
        /// Provider identifier.
        provider: String,
        /// Deterministic backend commit id.
        commit_id: String,
        /// Parsed receipt finality.
        finality: ReceiptFinality,
    },
    /// Provider detected duplicate idempotency key.
    Duplicate {
        /// Provider identifier.
        provider: String,
        /// Deterministic backend commit id.
        commit_id: String,
        /// Parsed receipt finality.
        finality: ReceiptFinality,
    },
    /// Provider rejected request with explicit reason.
    Rejected {
        /// Rejection reason.
        reason: String,
    },
}

/// Typed provider outcome extracted from one live response payload and normalized
/// to runtime receipt finality variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeRuntimeProviderOutcome {
    /// Provider accepted a new request submission.
    Submitted {
        /// Provider identifier.
        provider: String,
        /// Deterministic backend commit id.
        commit_id: String,
        /// Runtime receipt finality.
        finality: KolmeCommitReceiptFinality,
    },
    /// Provider detected duplicate idempotency key.
    Duplicate {
        /// Provider identifier.
        provider: String,
        /// Deterministic backend commit id.
        commit_id: String,
        /// Runtime receipt finality.
        finality: KolmeCommitReceiptFinality,
    },
    /// Provider rejected request with explicit reason.
    Rejected {
        /// Rejection reason.
        reason: String,
    },
}

/// Error returned by provider outcome parser and commit-id helper contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeProviderOutcomePolicyError {
    /// Response payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeProviderOutcomePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeProviderOutcomePolicyError {}

/// Error returned by provider receipt identity validation contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeProviderReceiptIdentityError {
    /// Provider identifier differs from expected runtime provider.
    ProviderMismatch {
        /// Expected provider identifier.
        expected: String,
        /// Observed provider identifier from receipt.
        observed: String,
    },
    /// Deterministic backend commit id is missing or empty.
    EmptyCommitId,
}

impl fmt::Display for KolmeProviderReceiptIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProviderMismatch { expected, observed } => write!(
                f,
                "provider mismatch: expected '{expected}' observed '{observed}'"
            ),
            Self::EmptyCommitId => f.write_str("receipt commit_id must not be empty"),
        }
    }
}

impl Error for KolmeProviderReceiptIdentityError {}

/// Parses a live provider response payload into one typed provider outcome.
pub fn parse_live_provider_outcome(
    response: &str,
    provider_hint: Option<&str>,
) -> Result<KolmeProviderOutcome, KolmeProviderOutcomePolicyError> {
    let fields = parse_provider_response_fields(response).map_err(map_provider_response_error)?;
    if let Some(status_raw) = fields.get("status") {
        let status = status_raw.trim();
        if status.is_empty() {
            return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: "field must not be empty: status".to_owned(),
            });
        }
        match status {
            "submitted" | "duplicate" => {
                let provider = required_response_field(&fields, "provider")?;
                let commit_id = resolve_commit_id(&fields)?;
                let finality_value = required_response_field(&fields, "finality")?;
                let finality =
                    parse_receipt_finality(finality_value.as_str()).map_err(map_finality_error)?;
                if status == "submitted" {
                    Ok(KolmeProviderOutcome::Submitted {
                        provider,
                        commit_id,
                        finality,
                    })
                } else {
                    Ok(KolmeProviderOutcome::Duplicate {
                        provider,
                        commit_id,
                        finality,
                    })
                }
            }
            "rejected" => {
                let reason = required_response_field(&fields, "reason")?;
                Ok(KolmeProviderOutcome::Rejected { reason })
            }
            _ => Err(KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: format!("invalid status value: {status}"),
            }),
        }
    } else {
        let has_tx_hash = fields.contains_key("txhash") || fields.contains_key("tx_hash");
        if !has_tx_hash {
            return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: "missing required field: status".to_owned(),
            });
        }

        let provider = optional_response_field(&fields, "provider").or_else(|| {
            provider_hint
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        });
        let provider =
            provider.ok_or_else(|| KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: "missing required field: provider".to_owned(),
            })?;
        let commit_id = resolve_commit_id(&fields)?;
        let finality = match optional_response_field(&fields, "finality") {
            Some(raw_finality) => {
                parse_receipt_finality(raw_finality.as_str()).map_err(map_finality_error)?
            }
            None => ReceiptFinality::Pending,
        };
        Ok(KolmeProviderOutcome::Submitted {
            provider,
            commit_id,
            finality,
        })
    }
}

/// Parses a live provider response payload and normalizes finality into runtime
/// receipt finality variants.
pub fn parse_live_runtime_provider_outcome(
    response: &str,
    provider_hint: Option<&str>,
) -> Result<KolmeRuntimeProviderOutcome, KolmeProviderOutcomePolicyError> {
    match parse_live_provider_outcome(response, provider_hint)? {
        KolmeProviderOutcome::Submitted {
            provider,
            commit_id,
            finality,
        } => Ok(KolmeRuntimeProviderOutcome::Submitted {
            provider,
            commit_id,
            finality: commit_finality_from_receipt_finality(finality),
        }),
        KolmeProviderOutcome::Duplicate {
            provider,
            commit_id,
            finality,
        } => Ok(KolmeRuntimeProviderOutcome::Duplicate {
            provider,
            commit_id,
            finality: commit_finality_from_receipt_finality(finality),
        }),
        KolmeProviderOutcome::Rejected { reason } => {
            Ok(KolmeRuntimeProviderOutcome::Rejected { reason })
        }
    }
}

/// Extracts one required non-empty field from provider response field maps.
pub fn required_provider_response_field(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Result<String, KolmeProviderOutcomePolicyError> {
    required_response_field(fields, field)
}

/// Resolves deterministic commit-id from provider response fields.
pub fn parse_commit_id_from_response_fields(
    fields: &HashMap<String, String>,
) -> Result<String, KolmeProviderOutcomePolicyError> {
    resolve_commit_id(fields)
}

/// Builds deterministic backend commit id from tx hash and optional block height.
pub fn deterministic_backend_commit_id(tx_hash: &str, block_height: Option<u64>) -> String {
    match block_height {
        Some(height) => format!("kolme-commit:{tx_hash}:h{height}"),
        None => format!("kolme-commit:{tx_hash}"),
    }
}

/// Parses tx hash segment out of deterministic backend commit id format.
pub fn txhash_from_commit_id(commit_id: &str) -> Result<String, KolmeProviderOutcomePolicyError> {
    let commit_id = commit_id.trim();
    if commit_id.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "commit_id must not be empty".to_owned(),
        });
    }
    let without_prefix = commit_id.strip_prefix("kolme-commit:").ok_or_else(|| {
        KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "commit_id must start with 'kolme-commit:'".to_owned(),
        }
    })?;

    let (txhash, block_suffix) = match without_prefix.split_once(":h") {
        Some((txhash, suffix)) => (txhash, Some(suffix)),
        None => (without_prefix, None),
    };
    let txhash = txhash.trim();
    if txhash.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "commit_id txhash segment must not be empty".to_owned(),
        });
    }

    if let Some(raw_height) = block_suffix {
        parse_block_height(raw_height)?;
    }

    Ok(txhash.to_owned())
}

/// Validates that deterministic commit id maps to the expected txhash value.
pub fn require_commit_id_matches_expected_txhash(
    commit_id: &str,
    expected_txhash: &str,
) -> Result<(), KolmeProviderOutcomePolicyError> {
    let expected_txhash = expected_txhash.trim();
    if expected_txhash.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "expected txhash must not be empty".to_owned(),
        });
    }
    let observed_txhash = txhash_from_commit_id(commit_id)?;
    if observed_txhash != expected_txhash {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: format!(
                "notification txhash mismatch: expected '{expected_txhash}' observed '{observed_txhash}'"
            ),
        });
    }
    Ok(())
}

/// Validates provider receipt identity tuple before adapter-level finality mapping.
pub fn validate_provider_receipt_identity(
    expected_provider: &str,
    observed_provider: &str,
    commit_id: &str,
) -> Result<(), KolmeProviderReceiptIdentityError> {
    if observed_provider != expected_provider {
        return Err(KolmeProviderReceiptIdentityError::ProviderMismatch {
            expected: expected_provider.to_owned(),
            observed: observed_provider.to_owned(),
        });
    }
    if commit_id.trim().is_empty() {
        return Err(KolmeProviderReceiptIdentityError::EmptyCommitId);
    }
    Ok(())
}

/// Validates adapter expected-provider input before receipt identity enforcement.
pub fn is_valid_expected_provider_input(expected_provider: &str) -> bool {
    is_valid_runtime_provider_input(expected_provider)
}

/// Validates runtime provider identifier input for provider-backed clients.
pub fn is_valid_runtime_provider_input(provider: &str) -> bool {
    !provider.trim().is_empty()
}

/// Validates provider hint input for `kolme_fork` submit profile configuration.
pub fn is_valid_provider_hint_input(provider_hint: &str) -> bool {
    is_valid_runtime_provider_input(provider_hint)
}

/// Validates receipt provider input for runtime lifecycle updates.
pub fn is_valid_receipt_provider_input(receipt_provider: &str) -> bool {
    is_valid_runtime_provider_input(receipt_provider)
}

/// Validates receipt commit identifier input for runtime lifecycle updates.
pub fn is_valid_receipt_commit_id_input(receipt_commit_id: &str) -> bool {
    !receipt_commit_id.trim().is_empty()
}

fn required_response_field(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Result<String, KolmeProviderOutcomePolicyError> {
    let value =
        fields
            .get(field)
            .ok_or_else(|| KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: format!("missing required field: {field}"),
            })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    Ok(trimmed.to_owned())
}

fn optional_response_field(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Option<String> {
    let value = fields.get(field)?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(trimmed.to_owned())
}

fn resolve_commit_id(
    fields: &HashMap<String, String>,
) -> Result<String, KolmeProviderOutcomePolicyError> {
    if let Some(commit_id) = fields.get("commit_id") {
        let trimmed = commit_id.trim();
        if trimmed.is_empty() {
            return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: "field must not be empty: commit_id".to_owned(),
            });
        }
        return Ok(trimmed.to_owned());
    }

    let tx_hash = fields
        .get("tx_hash")
        .or_else(|| fields.get("txhash"))
        .ok_or_else(|| KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "missing required field: commit_id".to_owned(),
        })?;
    let tx_hash = tx_hash.trim();
    if tx_hash.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "field must not be empty: tx_hash".to_owned(),
        });
    }

    let block_height = match fields.get("block_height") {
        Some(raw_height) => Some(parse_block_height(raw_height.as_str())?),
        None => None,
    };
    Ok(deterministic_backend_commit_id(tx_hash, block_height))
}

fn parse_block_height(raw: &str) -> Result<u64, KolmeProviderOutcomePolicyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "field must not be empty: block_height".to_owned(),
        });
    }
    let height =
        trimmed
            .parse::<u64>()
            .map_err(|_| KolmeProviderOutcomePolicyError::MalformedResponse {
                reason: format!("invalid block_height value: {trimmed}"),
            })?;
    if height == 0 {
        return Err(KolmeProviderOutcomePolicyError::MalformedResponse {
            reason: "block_height must be positive".to_owned(),
        });
    }
    Ok(height)
}

fn map_provider_response_error(
    error: KolmeProviderResponsePolicyError,
) -> KolmeProviderOutcomePolicyError {
    match error {
        KolmeProviderResponsePolicyError::MalformedResponse { reason } => {
            KolmeProviderOutcomePolicyError::MalformedResponse { reason }
        }
    }
}

fn map_finality_error(error: ReceiptFinalityError) -> KolmeProviderOutcomePolicyError {
    KolmeProviderOutcomePolicyError::MalformedResponse {
        reason: error.to_string(),
    }
}
