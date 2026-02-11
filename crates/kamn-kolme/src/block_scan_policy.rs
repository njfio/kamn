//! Block-scan policy contracts for Kolme fallback reconciliation.

use crate::provider_outcome_policy::deterministic_backend_commit_id;
use crate::receipt_finality::ReceiptFinality;
use std::error::Error;
use std::fmt;

/// Error raised by block-scan policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockScanPolicyError {
    /// `block_path_template` is empty.
    EmptyBlockPathTemplate,
    /// `block_path_template` is missing `{height}` placeholder.
    MissingHeightPlaceholder,
    /// `from_height` must be positive.
    NonPositiveFromHeight,
    /// `latest_height` must be positive.
    NonPositiveLatestHeight,
    /// `latest_height` must be >= `from_height`.
    LatestBeforeFromHeight,
    /// Block lookup span exceeds configured max.
    MaxLookupsExceeded {
        /// Lower bound of lookup window.
        from_height: u64,
        /// Upper bound of lookup window.
        latest_height: u64,
        /// Configured max lookups.
        max_lookups: u64,
    },
    /// Observed provider does not match configured provider.
    ProviderMismatch {
        /// Expected provider value.
        expected: String,
        /// Observed provider value.
        observed: String,
    },
    /// Observed block height does not match requested height.
    HeightMismatch {
        /// Expected block height.
        expected: u64,
        /// Observed block height.
        observed: u64,
    },
    /// Fork block-fallback response is malformed.
    MalformedForkFallbackResponse {
        /// Deterministic parse/validation failure reason.
        reason: String,
    },
}

impl fmt::Display for BlockScanPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBlockPathTemplate => f.write_str("block_path_template must not be empty"),
            Self::MissingHeightPlaceholder => {
                f.write_str("block_path_template must include '{height}' placeholder")
            }
            Self::NonPositiveFromHeight => f.write_str("from_height must be positive"),
            Self::NonPositiveLatestHeight => f.write_str("latest_height must be positive"),
            Self::LatestBeforeFromHeight => {
                f.write_str("latest_height must be greater than or equal to from_height")
            }
            Self::MaxLookupsExceeded {
                from_height,
                latest_height,
                max_lookups,
            } => write!(
                f,
                "block fallback window exceeds max lookups: from_height={from_height} latest_height={latest_height} max_lookups={max_lookups}"
            ),
            Self::ProviderMismatch { expected, observed } => write!(
                f,
                "block fallback provider mismatch: expected {expected} observed {observed}"
            ),
            Self::HeightMismatch { expected, observed } => write!(
                f,
                "block fallback response height mismatch: expected {expected} observed {observed}"
            ),
            Self::MalformedForkFallbackResponse { reason } => f.write_str(reason),
        }
    }
}

impl Error for BlockScanPolicyError {}

/// Deterministic receipt projection emitted from one block txhash match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockScanReceiptProjection {
    /// Deterministic backend commit id.
    pub commit_id: String,
    /// Projected receipt finality.
    pub finality: ReceiptFinality,
}

/// Validates the configured block endpoint path template.
pub fn validate_block_path_template(block_path_template: &str) -> Result<(), BlockScanPolicyError> {
    let trimmed = block_path_template.trim();
    if trimmed.is_empty() {
        return Err(BlockScanPolicyError::EmptyBlockPathTemplate);
    }
    if !trimmed.contains("{height}") {
        return Err(BlockScanPolicyError::MissingHeightPlaceholder);
    }
    Ok(())
}

/// Renders one block lookup path from template and concrete height.
pub fn render_block_path(
    block_path_template: &str,
    height: u64,
) -> Result<String, BlockScanPolicyError> {
    validate_block_path_template(block_path_template)?;
    Ok(block_path_template
        .trim()
        .replace("{height}", height.to_string().as_str()))
}

/// Validates one bounded block lookup window.
pub fn validate_lookup_window(
    from_height: u64,
    latest_height: u64,
    max_lookups: u64,
) -> Result<(), BlockScanPolicyError> {
    if from_height == 0 {
        return Err(BlockScanPolicyError::NonPositiveFromHeight);
    }
    if latest_height == 0 {
        return Err(BlockScanPolicyError::NonPositiveLatestHeight);
    }
    if latest_height < from_height {
        return Err(BlockScanPolicyError::LatestBeforeFromHeight);
    }
    let lookup_span = latest_height - from_height + 1;
    if lookup_span > max_lookups {
        return Err(BlockScanPolicyError::MaxLookupsExceeded {
            from_height,
            latest_height,
            max_lookups,
        });
    }
    Ok(())
}

/// Resolves block-fallback upper bound using one latest-block notification height.
pub fn resolve_lookup_upper_bound(
    from_height: u64,
    latest_height: u64,
    notification_height: u64,
) -> u64 {
    if notification_height >= from_height {
        notification_height.min(latest_height)
    } else {
        latest_height
    }
}

/// Projects a finalized block match into deterministic receipt projection fields.
pub fn project_finalized_block_txhash_receipt(
    txhash: &str,
    block_height: u64,
) -> BlockScanReceiptProjection {
    BlockScanReceiptProjection {
        commit_id: deterministic_backend_commit_id(txhash, Some(block_height)),
        finality: ReceiptFinality::Final,
    }
}

/// Projects a failed block match into deterministic receipt projection fields.
pub fn project_failed_block_txhash_receipt(txhash: &str) -> BlockScanReceiptProjection {
    BlockScanReceiptProjection {
        commit_id: deterministic_backend_commit_id(txhash, None),
        finality: ReceiptFinality::Failed,
    }
}

/// Validates provider + height identity for one scanned block response.
pub fn validate_block_identity(
    expected_provider: &str,
    observed_provider: &str,
    expected_height: u64,
    observed_height: u64,
) -> Result<(), BlockScanPolicyError> {
    if observed_provider != expected_provider {
        return Err(BlockScanPolicyError::ProviderMismatch {
            expected: expected_provider.to_owned(),
            observed: observed_provider.to_owned(),
        });
    }
    if observed_height != expected_height {
        return Err(BlockScanPolicyError::HeightMismatch {
            expected: expected_height,
            observed: observed_height,
        });
    }
    Ok(())
}

/// Parses the required `txhash` field from a fork block-fallback payload.
pub fn parse_fork_block_txhash(response: &str) -> Result<String, BlockScanPolicyError> {
    find_string_field(response, "txhash")?.ok_or_else(|| {
        BlockScanPolicyError::MalformedForkFallbackResponse {
            reason: "missing required field: txhash".to_owned(),
        }
    })
}

fn find_string_field(payload: &str, field: &str) -> Result<Option<String>, BlockScanPolicyError> {
    let pattern = format!("\"{field}\"");
    for (index, _) in payload.match_indices(pattern.as_str()) {
        let mut cursor = index + pattern.len();
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b':') {
            continue;
        }
        cursor += 1;
        cursor = skip_ascii_whitespace(payload, cursor);
        if payload.as_bytes().get(cursor).copied() != Some(b'"') {
            continue;
        }

        let mut end = cursor + 1;
        let mut escape = false;
        while let Some(byte) = payload.as_bytes().get(end).copied() {
            if escape {
                escape = false;
                end += 1;
                continue;
            }
            if byte == b'\\' {
                escape = true;
                end += 1;
                continue;
            }
            if byte == b'"' {
                let token = &payload[cursor..=end];
                let parsed = parse_json_string(token).map_err(|reason| {
                    BlockScanPolicyError::MalformedForkFallbackResponse {
                        reason: format!("notification field '{field}' is invalid: {reason}"),
                    }
                })?;
                if parsed.trim().is_empty() {
                    return Err(BlockScanPolicyError::MalformedForkFallbackResponse {
                        reason: format!("notification field '{field}' must not be empty"),
                    });
                }
                return Ok(Some(parsed));
            }
            end += 1;
        }
        return Err(BlockScanPolicyError::MalformedForkFallbackResponse {
            reason: format!("notification field '{field}' is unterminated"),
        });
    }
    Ok(None)
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

fn skip_ascii_whitespace(value: &str, mut cursor: usize) -> usize {
    while let Some(byte) = value.as_bytes().get(cursor).copied() {
        if byte.is_ascii_whitespace() {
            cursor += 1;
            continue;
        }
        break;
    }
    cursor
}

#[cfg(test)]
mod tests {
    use super::{
        parse_fork_block_txhash, render_block_path, validate_block_identity,
        validate_block_path_template, validate_lookup_window, BlockScanPolicyError,
    };

    #[test]
    fn unit_validate_block_path_template_requires_height_placeholder() {
        assert_eq!(
            validate_block_path_template("/block/static"),
            Err(BlockScanPolicyError::MissingHeightPlaceholder)
        );
        assert!(validate_block_path_template("/block/{height}").is_ok());
    }

    #[test]
    fn unit_validate_lookup_window_requires_non_stale_bounds() {
        assert_eq!(
            validate_lookup_window(5, 4, 3),
            Err(BlockScanPolicyError::LatestBeforeFromHeight)
        );
    }

    #[test]
    fn unit_validate_block_identity_rejects_mismatch() {
        assert_eq!(
            validate_block_identity("kolme-a", "kolme-b", 42, 42),
            Err(BlockScanPolicyError::ProviderMismatch {
                expected: "kolme-a".to_owned(),
                observed: "kolme-b".to_owned(),
            })
        );
        assert_eq!(
            render_block_path("/block/{height}", 42).expect("render should pass"),
            "/block/42"
        );
    }

    #[test]
    fn unit_parse_fork_block_txhash_rejects_empty_txhash() {
        assert_eq!(
            parse_fork_block_txhash(r#"{"txhash":"   "}"#),
            Err(BlockScanPolicyError::MalformedForkFallbackResponse {
                reason: "notification field 'txhash' must not be empty".to_owned(),
            })
        );
    }
}
