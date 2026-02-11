//! Block-fallback response parsing contracts for Kolme reconciliation flows.

use crate::{
    parse_flat_json_value_fields, parse_fork_block_txhash, parse_provider_response_fields,
    required_json_string_field, required_positive_u64_json_field, BlockScanPolicyError,
    KolmeFlatJsonPolicyError, KolmeFlatJsonValue, KolmeProviderResponsePolicyError,
};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Parsed block-fallback response used by reconciliation paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KolmeBlockFallbackResponse {
    /// Provider identifier.
    pub provider: String,
    /// Block height for this lookup response.
    pub block_height: u64,
    /// Finalized tx hash candidates observed in this block.
    pub finalized_tx_hashes: Vec<String>,
    /// Failed tx hash candidates observed in this block.
    pub failed_tx_hashes: Vec<String>,
}

/// Error returned by block-fallback parsing contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeBlockFallbackPolicyError {
    /// Response payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeBlockFallbackPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeBlockFallbackPolicyError {}

/// Parses one block-fallback response payload into typed fallback fields.
pub fn parse_block_fallback_response(
    response: &str,
) -> Result<KolmeBlockFallbackResponse, KolmeBlockFallbackPolicyError> {
    let trimmed = response.trim();
    if trimmed.starts_with('{') {
        let fields = parse_flat_json_value_fields(trimmed).map_err(map_flat_json_error)?;
        let provider =
            required_json_string_field(&fields, "provider").map_err(map_flat_json_error)?;
        let block_height = required_block_height_json_field(&fields)?;
        let finalized_tx_hashes = optional_json_block_tx_hashes(&fields, "tx_hashes")?;
        let failed_tx_hashes = optional_json_block_tx_hashes(&fields, "failed_tx_hashes")?;
        return Ok(KolmeBlockFallbackResponse {
            provider,
            block_height,
            finalized_tx_hashes,
            failed_tx_hashes,
        });
    }

    let fields = parse_provider_response_fields(trimmed).map_err(map_provider_response_error)?;
    let provider = required_response_field(&fields, "provider")?;
    let block_height = fields
        .get("block_height")
        .or_else(|| fields.get("height"))
        .ok_or_else(|| KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "missing required field: block_height".to_owned(),
        })
        .and_then(|value| parse_block_height(value.as_str()))?;
    let finalized_tx_hashes = optional_block_tx_hashes(&fields, "tx_hashes")?;
    let failed_tx_hashes = optional_block_tx_hashes(&fields, "failed_tx_hashes")?;
    Ok(KolmeBlockFallbackResponse {
        provider,
        block_height,
        finalized_tx_hashes,
        failed_tx_hashes,
    })
}

/// Parses fork block fallback payload and maps txhash to finalized fallback fields.
pub fn parse_fork_block_fallback_response(
    response: &str,
    provider: &str,
    expected_height: u64,
) -> Result<KolmeBlockFallbackResponse, KolmeBlockFallbackPolicyError> {
    let provider = provider.trim();
    if provider.is_empty() {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "provider must not be empty".to_owned(),
        });
    }
    if expected_height == 0 {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "expected block height must be positive".to_owned(),
        });
    }

    let txhash = parse_fork_block_txhash(response).map_err(map_block_scan_error)?;
    Ok(KolmeBlockFallbackResponse {
        provider: provider.to_owned(),
        block_height: expected_height,
        finalized_tx_hashes: vec![txhash],
        failed_tx_hashes: Vec::new(),
    })
}

fn optional_block_tx_hashes(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Result<Vec<String>, KolmeBlockFallbackPolicyError> {
    let Some(value) = fields.get(field) else {
        return Ok(Vec::new());
    };
    parse_tx_hash_list(value, field)
}

fn parse_tx_hash_list(
    raw: &str,
    field: &'static str,
) -> Result<Vec<String>, KolmeBlockFallbackPolicyError> {
    if raw.trim().is_empty() {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    let mut values = Vec::new();
    for token in raw.split(',') {
        let txhash = token.trim();
        if txhash.is_empty() {
            return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
                reason: format!("field contains empty tx hash token: {field}"),
            });
        }
        values.push(txhash.to_owned());
    }
    Ok(values)
}

fn required_block_height_json_field(
    fields: &HashMap<String, KolmeFlatJsonValue>,
) -> Result<u64, KolmeBlockFallbackPolicyError> {
    if fields.contains_key("block_height") {
        return required_positive_u64_json_field(fields, "block_height")
            .map_err(map_flat_json_error);
    }
    if fields.contains_key("height") {
        return required_positive_u64_json_field(fields, "height").map_err(map_flat_json_error);
    }
    Err(KolmeBlockFallbackPolicyError::MalformedResponse {
        reason: "missing required field: block_height".to_owned(),
    })
}

fn optional_json_block_tx_hashes(
    fields: &HashMap<String, KolmeFlatJsonValue>,
    field: &'static str,
) -> Result<Vec<String>, KolmeBlockFallbackPolicyError> {
    let Some(value) = fields.get(field) else {
        return Ok(Vec::new());
    };
    match value {
        KolmeFlatJsonValue::Null => Ok(Vec::new()),
        KolmeFlatJsonValue::String(raw) => parse_tx_hash_list(raw, field),
        _ => Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: format!("field must be string|null: {field}"),
        }),
    }
}

fn required_response_field(
    fields: &HashMap<String, String>,
    field: &'static str,
) -> Result<String, KolmeBlockFallbackPolicyError> {
    let value =
        fields
            .get(field)
            .ok_or_else(|| KolmeBlockFallbackPolicyError::MalformedResponse {
                reason: format!("missing required field: {field}"),
            })?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    Ok(trimmed.to_owned())
}

fn parse_block_height(raw: &str) -> Result<u64, KolmeBlockFallbackPolicyError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "field must not be empty: block_height".to_owned(),
        });
    }
    let height =
        trimmed
            .parse::<u64>()
            .map_err(|_| KolmeBlockFallbackPolicyError::MalformedResponse {
                reason: format!("invalid block_height value: {trimmed}"),
            })?;
    if height == 0 {
        return Err(KolmeBlockFallbackPolicyError::MalformedResponse {
            reason: "block_height must be positive".to_owned(),
        });
    }
    Ok(height)
}

fn map_block_scan_error(error: BlockScanPolicyError) -> KolmeBlockFallbackPolicyError {
    KolmeBlockFallbackPolicyError::MalformedResponse {
        reason: error.to_string(),
    }
}

fn map_provider_response_error(
    error: KolmeProviderResponsePolicyError,
) -> KolmeBlockFallbackPolicyError {
    match error {
        KolmeProviderResponsePolicyError::MalformedResponse { reason } => {
            KolmeBlockFallbackPolicyError::MalformedResponse { reason }
        }
    }
}

fn map_flat_json_error(error: KolmeFlatJsonPolicyError) -> KolmeBlockFallbackPolicyError {
    match error {
        KolmeFlatJsonPolicyError::MalformedResponse { reason } => {
            KolmeBlockFallbackPolicyError::MalformedResponse { reason }
        }
    }
}
