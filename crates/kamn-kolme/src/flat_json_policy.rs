//! Flat JSON scalar/object parsing contracts for Kolme runtime-commit payloads.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Typed scalar value parsed from a flat JSON object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeFlatJsonValue {
    /// JSON string value.
    String(String),
    /// JSON integer number literal.
    Number(String),
    /// JSON boolean value.
    Boolean(bool),
    /// JSON null value.
    Null,
}

/// Error raised by flat JSON parser policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KolmeFlatJsonPolicyError {
    /// JSON payload failed deterministic parse/validation.
    MalformedResponse {
        /// Parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for KolmeFlatJsonPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for KolmeFlatJsonPolicyError {}

/// Parses a flat JSON object into scalar field values.
pub fn parse_flat_json_value_fields(
    response: &str,
) -> Result<HashMap<String, KolmeFlatJsonValue>, KolmeFlatJsonPolicyError> {
    let body = response.trim();
    if !(body.starts_with('{') && body.ends_with('}')) {
        return Err(KolmeFlatJsonPolicyError::MalformedResponse {
            reason: "json response must be an object".to_owned(),
        });
    }
    let inner = &body[1..body.len() - 1];
    if inner.trim().is_empty() {
        return Ok(HashMap::new());
    }

    let entries = split_unquoted(inner, ',').map_err(|reason| {
        KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("invalid json response: {reason}"),
        }
    })?;

    let mut fields = HashMap::new();
    for entry in entries {
        let pair = split_unquoted(entry.as_str(), ':').map_err(|reason| {
            KolmeFlatJsonPolicyError::MalformedResponse {
                reason: format!("invalid json response pair: {reason}"),
            }
        })?;
        if pair.len() != 2 {
            return Err(KolmeFlatJsonPolicyError::MalformedResponse {
                reason: "json response pair must contain exactly one ':'".to_owned(),
            });
        }

        let key = parse_json_string(pair[0].trim()).map_err(|reason| {
            KolmeFlatJsonPolicyError::MalformedResponse {
                reason: format!("invalid json key: {reason}"),
            }
        })?;
        let value = parse_json_value(pair[1].trim())?;
        fields.insert(key, value);
    }
    Ok(fields)
}

/// Extracts one non-empty string field from parsed flat JSON fields.
pub fn required_json_string_field(
    fields: &HashMap<String, KolmeFlatJsonValue>,
    field: &'static str,
) -> Result<String, KolmeFlatJsonPolicyError> {
    let value = fields
        .get(field)
        .ok_or_else(|| KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("missing required field: {field}"),
        })?;
    let KolmeFlatJsonValue::String(raw) = value else {
        return Err(KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("field must be a string: {field}"),
        });
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("field must not be empty: {field}"),
        });
    }
    Ok(trimmed.to_owned())
}

/// Extracts one positive `u64` field from parsed flat JSON fields.
pub fn required_positive_u64_json_field(
    fields: &HashMap<String, KolmeFlatJsonValue>,
    field: &'static str,
) -> Result<u64, KolmeFlatJsonPolicyError> {
    let value = fields
        .get(field)
        .ok_or_else(|| KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("missing required field: {field}"),
        })?;

    let raw = match value {
        KolmeFlatJsonValue::Number(value) => value.as_str(),
        KolmeFlatJsonValue::String(value) => value.trim(),
        _ => {
            return Err(KolmeFlatJsonPolicyError::MalformedResponse {
                reason: format!("field must be numeric: {field}"),
            });
        }
    };

    let parsed = raw
        .parse::<i128>()
        .map_err(|_| KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("invalid numeric field: {field}"),
        })?;
    if parsed <= 0 {
        return Err(KolmeFlatJsonPolicyError::MalformedResponse {
            reason: format!("{field} must be positive"),
        });
    }
    u64::try_from(parsed).map_err(|_| KolmeFlatJsonPolicyError::MalformedResponse {
        reason: format!("invalid numeric field: {field}"),
    })
}

fn parse_json_value(token: &str) -> Result<KolmeFlatJsonValue, KolmeFlatJsonPolicyError> {
    let trimmed = token.trim();
    if trimmed.starts_with('"') {
        let value = parse_json_string(trimmed).map_err(|reason| {
            KolmeFlatJsonPolicyError::MalformedResponse {
                reason: format!("invalid json value: {reason}"),
            }
        })?;
        return Ok(KolmeFlatJsonValue::String(value));
    }
    if trimmed == "null" {
        return Ok(KolmeFlatJsonValue::Null);
    }
    if trimmed == "true" {
        return Ok(KolmeFlatJsonValue::Boolean(true));
    }
    if trimmed == "false" {
        return Ok(KolmeFlatJsonValue::Boolean(false));
    }
    if is_json_number_literal(trimmed) {
        return Ok(KolmeFlatJsonValue::Number(trimmed.to_owned()));
    }
    Err(KolmeFlatJsonPolicyError::MalformedResponse {
        reason: "invalid json value token".to_owned(),
    })
}

fn is_json_number_literal(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let mut chars = token.chars();
    let first = chars.next().unwrap_or_default();
    if first == '-' {
        let remainder = chars.as_str();
        return !remainder.is_empty() && remainder.chars().all(|ch| ch.is_ascii_digit());
    }
    first.is_ascii_digit() && chars.as_str().chars().all(|ch| ch.is_ascii_digit())
}

fn split_unquoted(input: &str, delimiter: char) -> Result<Vec<String>, &'static str> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' && in_quotes {
            current.push(ch);
            escape = true;
            continue;
        }

        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
            continue;
        }

        if ch == delimiter && !in_quotes {
            if current.trim().is_empty() {
                return Err("empty segment");
            }
            parts.push(current.trim().to_owned());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if in_quotes {
        return Err("unterminated quoted string");
    }
    if current.trim().is_empty() {
        return Err("empty trailing segment");
    }
    parts.push(current.trim().to_owned());
    Ok(parts)
}

fn parse_json_string(token: &str) -> Result<String, &'static str> {
    let trimmed = token.trim();
    if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
        return Err("token must be a quoted string");
    }
    let mut output = String::new();
    let mut escape = false;
    for ch in trimmed[1..trimmed.len() - 1].chars() {
        if escape {
            let mapped = match ch {
                '\\' => '\\',
                '"' => '"',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                _ => return Err("unsupported escape sequence"),
            };
            output.push(mapped);
            escape = false;
            continue;
        }
        if ch == '\\' {
            escape = true;
            continue;
        }
        output.push(ch);
    }
    if escape {
        return Err("unterminated escape sequence");
    }
    Ok(output)
}
