//! Block-scan policy contracts for Kolme fallback reconciliation.

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
        }
    }
}

impl Error for BlockScanPolicyError {}

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

#[cfg(test)]
mod tests {
    use super::{
        render_block_path, validate_block_identity, validate_block_path_template,
        validate_lookup_window, BlockScanPolicyError,
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
}
