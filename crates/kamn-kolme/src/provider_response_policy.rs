//! Provider response parsing contracts for Kolme runtime-commit API calls.

use crate::json_parse_helpers::split_unquoted;
use crate::json_scalar_policy::parse_json_string_token as parse_json_string;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Error raised while parsing provider response payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeProviderResponsePolicyError {
    /// Provider payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeProviderResponsePolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeProviderResponsePolicyError {}

/// Parses provider response payload fields from key/value or flat JSON formats.
pub fn parse_provider_response_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeProviderResponsePolicyError> {
    let trimmed = response.trim();
    if trimmed.is_empty() {
        return Err(KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "response body must not be empty".to_owned(),
        });
    }

    if trimmed.starts_with('{') {
        return parse_flat_json_response_fields(trimmed);
    }

    parse_provider_key_value_fields(trimmed)
}

/// Parses a line-delimited key/value response payload into field pairs.
pub fn parse_provider_key_value_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeProviderResponsePolicyError> {
    let mut fields = HashMap::new();
    for line in response.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            KolmeProviderResponsePolicyError::MalformedResponse {
                reason: format!("invalid key/value response line: {trimmed}"),
            }
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            return Err(KolmeProviderResponsePolicyError::MalformedResponse {
                reason: format!("invalid key/value response line: {trimmed}"),
            });
        }
        fields.insert(key.to_owned(), value.to_owned());
    }
    if fields.is_empty() {
        return Err(KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "response body must contain at least one field".to_owned(),
        });
    }
    Ok(fields)
}

fn parse_flat_json_response_fields(
    response: &str,
) -> Result<HashMap<String, String>, KolmeProviderResponsePolicyError> {
    let body = response.trim();
    if !(body.starts_with('{') && body.ends_with('}')) {
        return Err(KolmeProviderResponsePolicyError::MalformedResponse {
            reason: "json response must be an object".to_owned(),
        });
    }
    let inner = &body[1..body.len() - 1];
    if inner.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let entries = split_unquoted(inner, ',').map_err(|reason| {
        KolmeProviderResponsePolicyError::MalformedResponse {
            reason: format!("invalid json response: {reason}"),
        }
    })?;

    let mut fields = HashMap::new();
    for entry in entries {
        let pair = split_unquoted(entry.as_str(), ':').map_err(|reason| {
            KolmeProviderResponsePolicyError::MalformedResponse {
                reason: format!("invalid json response pair: {reason}"),
            }
        })?;
        if pair.len() != 2 {
            return Err(KolmeProviderResponsePolicyError::MalformedResponse {
                reason: "json response pair must contain exactly one ':'".to_owned(),
            });
        }

        let key = parse_json_string(pair[0].trim()).map_err(|reason| {
            KolmeProviderResponsePolicyError::MalformedResponse {
                reason: format!("invalid json key: {reason}"),
            }
        })?;
        let value = parse_json_string(pair[1].trim()).map_err(|reason| {
            KolmeProviderResponsePolicyError::MalformedResponse {
                reason: format!("invalid json value: {reason}"),
            }
        })?;
        fields.insert(key, value);
    }
    Ok(fields)
}
