//! Token configuration and genesis allocation validation contracts.

use std::collections::HashSet;
use std::fmt;

/// Default token symbol.
pub const DEFAULT_TOKEN_SYMBOL: &str = "KAMN";
/// Default total token supply.
pub const DEFAULT_TOTAL_SUPPLY: u128 = 1_000_000_000;
/// Default token decimal precision.
pub const DEFAULT_DECIMALS: u8 = 18;
/// Basis-point total expected across all genesis allocation buckets.
pub const TOTAL_ALLOCATION_BPS: u16 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Logical bucket for genesis token allocation.
pub enum AllocationBucket {
    /// Ecosystem and growth incentives allocation.
    EcosystemIncentives,
    /// Protocol development and maintenance allocation.
    ProtocolDevelopment,
    /// Validator and network participation rewards allocation.
    ValidatorRewards,
    /// Initial liquidity provisioning allocation.
    InitialLiquidity,
    /// Community grants and public goods allocation.
    CommunityGrants,
}

impl AllocationBucket {
    /// Returns a stable snake_case identifier for the bucket.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EcosystemIncentives => "ecosystem_incentives",
            Self::ProtocolDevelopment => "protocol_development",
            Self::ValidatorRewards => "validator_rewards",
            Self::InitialLiquidity => "initial_liquidity",
            Self::CommunityGrants => "community_grants",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Genesis allocation entry for one allocation bucket.
pub struct GenesisAllocation {
    /// Allocation bucket identifier.
    pub bucket: AllocationBucket,
    /// Share in basis points.
    pub share_bps: u16,
    /// Absolute token amount assigned to the bucket.
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Token configuration including supply, precision, and allocations.
pub struct TokenConfig {
    /// Token symbol.
    pub symbol: String,
    /// Total token supply.
    pub total_supply: u128,
    /// Token decimal precision.
    pub decimals: u8,
    /// Genesis allocation table.
    pub allocations: Vec<GenesisAllocation>,
}

impl TokenConfig {
    /// Returns the allocation entry for `bucket`, if present.
    pub fn allocation_for(&self, bucket: AllocationBucket) -> Option<&GenesisAllocation> {
        self.allocations.iter().find(|item| item.bucket == bucket)
    }

    /// Validates symbol, supply, decimals, and allocation invariants.
    pub fn validate(&self) -> Result<(), TokenConfigError> {
        if self.symbol.trim().is_empty() {
            return Err(TokenConfigError::EmptySymbol);
        }
        if !self
            .symbol
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
        {
            return Err(TokenConfigError::InvalidSymbol(self.symbol.clone()));
        }
        if self.total_supply == 0 {
            return Err(TokenConfigError::ZeroSupply);
        }
        if self.decimals > DEFAULT_DECIMALS {
            return Err(TokenConfigError::InvalidDecimals(self.decimals));
        }
        if self.allocations.is_empty() {
            return Err(TokenConfigError::EmptyAllocations);
        }

        let mut bucket_set: HashSet<AllocationBucket> = HashSet::new();
        let mut share_sum: u32 = 0;
        let mut amount_sum: u128 = 0;
        for allocation in &self.allocations {
            if allocation.share_bps == 0 {
                return Err(TokenConfigError::ZeroShare(allocation.bucket));
            }
            if allocation.amount == 0 {
                return Err(TokenConfigError::ZeroAmount(allocation.bucket));
            }
            if !bucket_set.insert(allocation.bucket) {
                return Err(TokenConfigError::DuplicateBucket(allocation.bucket));
            }
            share_sum += u32::from(allocation.share_bps);
            amount_sum = amount_sum
                .checked_add(allocation.amount)
                .ok_or(TokenConfigError::AmountOverflow)?;
        }

        if share_sum != u32::from(TOTAL_ALLOCATION_BPS) {
            return Err(TokenConfigError::AllocationShareSum {
                expected: TOTAL_ALLOCATION_BPS,
                actual: share_sum as u16,
            });
        }
        if amount_sum != self.total_supply {
            return Err(TokenConfigError::AllocationAmountSum {
                expected: self.total_supply,
                actual: amount_sum,
            });
        }

        Ok(())
    }
}

/// Returns the default token configuration.
pub fn default_token_config() -> TokenConfig {
    TokenConfig {
        symbol: DEFAULT_TOKEN_SYMBOL.to_owned(),
        total_supply: DEFAULT_TOTAL_SUPPLY,
        decimals: DEFAULT_DECIMALS,
        allocations: vec![
            GenesisAllocation {
                bucket: AllocationBucket::EcosystemIncentives,
                share_bps: 4_000,
                amount: 400_000_000,
            },
            GenesisAllocation {
                bucket: AllocationBucket::ProtocolDevelopment,
                share_bps: 2_500,
                amount: 250_000_000,
            },
            GenesisAllocation {
                bucket: AllocationBucket::ValidatorRewards,
                share_bps: 2_000,
                amount: 200_000_000,
            },
            GenesisAllocation {
                bucket: AllocationBucket::InitialLiquidity,
                share_bps: 1_000,
                amount: 100_000_000,
            },
            GenesisAllocation {
                bucket: AllocationBucket::CommunityGrants,
                share_bps: 500,
                amount: 50_000_000,
            },
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for token configuration validation failures.
pub enum TokenConfigError {
    /// Token symbol is empty after trimming.
    EmptySymbol,
    /// Token symbol contains invalid characters or casing.
    InvalidSymbol(String),
    /// Total supply is zero.
    ZeroSupply,
    /// Decimals exceed the supported precision.
    InvalidDecimals(u8),
    /// Allocation table is empty.
    EmptyAllocations,
    /// Allocation bucket appears multiple times.
    DuplicateBucket(AllocationBucket),
    /// Allocation share is zero.
    ZeroShare(AllocationBucket),
    /// Allocation amount is zero.
    ZeroAmount(AllocationBucket),
    /// Allocation share sum does not match total basis points.
    AllocationShareSum {
        /// Expected basis-point sum.
        expected: u16,
        /// Observed basis-point sum.
        actual: u16,
    },
    /// Allocation amount sum does not match total supply.
    AllocationAmountSum {
        /// Expected allocation amount sum.
        expected: u128,
        /// Observed allocation amount sum.
        actual: u128,
    },
    /// Allocation amount accumulation overflowed.
    AmountOverflow,
}

impl fmt::Display for TokenConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySymbol => write!(f, "token symbol must not be empty"),
            Self::InvalidSymbol(value) => write!(f, "token symbol must be uppercase: {value}"),
            Self::ZeroSupply => write!(f, "total token supply must be greater than zero"),
            Self::InvalidDecimals(value) => {
                write!(
                    f,
                    "token decimals must be <= {DEFAULT_DECIMALS}, got {value}"
                )
            }
            Self::EmptyAllocations => write!(f, "genesis allocations must not be empty"),
            Self::DuplicateBucket(bucket) => {
                write!(f, "duplicate allocation bucket: {}", bucket.as_str())
            }
            Self::ZeroShare(bucket) => {
                write!(f, "allocation share must be > 0: {}", bucket.as_str())
            }
            Self::ZeroAmount(bucket) => {
                write!(f, "allocation amount must be > 0: {}", bucket.as_str())
            }
            Self::AllocationShareSum { expected, actual } => {
                write!(
                    f,
                    "allocation share bps mismatch, expected {expected}, got {actual}"
                )
            }
            Self::AllocationAmountSum { expected, actual } => {
                write!(
                    f,
                    "allocation amount sum mismatch, expected {expected}, got {actual}"
                )
            }
            Self::AmountOverflow => write!(f, "allocation amount overflow"),
        }
    }
}

impl std::error::Error for TokenConfigError {}

#[cfg(test)]
mod tests {
    use super::{
        default_token_config, AllocationBucket, TokenConfigError, DEFAULT_TOKEN_SYMBOL,
        DEFAULT_TOTAL_SUPPLY,
    };

    #[test]
    fn default_model_validates() {
        let config = default_token_config();
        assert_eq!(config.symbol, DEFAULT_TOKEN_SYMBOL);
        assert_eq!(config.total_supply, DEFAULT_TOTAL_SUPPLY);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn detects_share_sum_mismatch() {
        let mut config = default_token_config();
        config.allocations[0].share_bps = 3_999;
        assert_eq!(
            config.validate(),
            Err(TokenConfigError::AllocationShareSum {
                expected: 10_000,
                actual: 9_999
            })
        );
    }

    #[test]
    fn allocation_lookup_by_bucket() {
        let config = default_token_config();
        assert_eq!(
            config
                .allocation_for(AllocationBucket::ValidatorRewards)
                .map(|value| value.amount),
            Some(200_000_000)
        );
    }
}
