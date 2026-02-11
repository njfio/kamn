//! Provider outcome parsing and commit-id helper contracts for runtime commits.

use crate::{
    parse_provider_response_fields, parse_receipt_finality, KolmeProviderResponsePolicyError,
    ReceiptFinality, ReceiptFinalityError,
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
