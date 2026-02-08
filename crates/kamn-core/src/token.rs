use std::collections::HashSet;
use std::fmt;

pub const DEFAULT_TOKEN_SYMBOL: &str = "KAMN";
pub const DEFAULT_TOTAL_SUPPLY: u128 = 1_000_000_000;
pub const DEFAULT_DECIMALS: u8 = 18;
pub const TOTAL_ALLOCATION_BPS: u16 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllocationBucket {
    EcosystemIncentives,
    ProtocolDevelopment,
    ValidatorRewards,
    InitialLiquidity,
    CommunityGrants,
}

impl AllocationBucket {
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
pub struct GenesisAllocation {
    pub bucket: AllocationBucket,
    pub share_bps: u16,
    pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenConfig {
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
    pub allocations: Vec<GenesisAllocation>,
}

impl TokenConfig {
    pub fn allocation_for(&self, bucket: AllocationBucket) -> Option<&GenesisAllocation> {
        self.allocations.iter().find(|item| item.bucket == bucket)
    }

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
pub enum TokenConfigError {
    EmptySymbol,
    InvalidSymbol(String),
    ZeroSupply,
    InvalidDecimals(u8),
    EmptyAllocations,
    DuplicateBucket(AllocationBucket),
    ZeroShare(AllocationBucket),
    ZeroAmount(AllocationBucket),
    AllocationShareSum { expected: u16, actual: u16 },
    AllocationAmountSum { expected: u128, actual: u128 },
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
