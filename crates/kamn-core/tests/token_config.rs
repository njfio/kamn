use kamn_core::{
    default_token_config, AllocationBucket, TokenConfig, TokenConfigError, DEFAULT_DECIMALS,
    DEFAULT_TOKEN_SYMBOL, DEFAULT_TOTAL_SUPPLY,
};

#[test]
fn default_token_config_matches_prd_token_model() {
    let config = default_token_config();

    assert_eq!(config.symbol, DEFAULT_TOKEN_SYMBOL);
    assert_eq!(config.total_supply, DEFAULT_TOTAL_SUPPLY);
    assert_eq!(config.decimals, DEFAULT_DECIMALS);

    assert_eq!(
        config
            .allocation_for(AllocationBucket::EcosystemIncentives)
            .map(|allocation| allocation.amount),
        Some(400_000_000)
    );
    assert_eq!(
        config
            .allocation_for(AllocationBucket::ProtocolDevelopment)
            .map(|allocation| allocation.amount),
        Some(250_000_000)
    );
    assert_eq!(
        config
            .allocation_for(AllocationBucket::ValidatorRewards)
            .map(|allocation| allocation.amount),
        Some(200_000_000)
    );
    assert_eq!(
        config
            .allocation_for(AllocationBucket::InitialLiquidity)
            .map(|allocation| allocation.amount),
        Some(100_000_000)
    );
    assert_eq!(
        config
            .allocation_for(AllocationBucket::CommunityGrants)
            .map(|allocation| allocation.amount),
        Some(50_000_000)
    );
}

#[test]
fn token_config_validate_rejects_duplicate_buckets() {
    let config = TokenConfig {
        symbol: DEFAULT_TOKEN_SYMBOL.to_owned(),
        total_supply: DEFAULT_TOTAL_SUPPLY,
        decimals: DEFAULT_DECIMALS,
        allocations: vec![
            kamn_core::GenesisAllocation {
                bucket: AllocationBucket::EcosystemIncentives,
                share_bps: 4_000,
                amount: 400_000_000,
            },
            kamn_core::GenesisAllocation {
                bucket: AllocationBucket::EcosystemIncentives,
                share_bps: 6_000,
                amount: 600_000_000,
            },
        ],
    };

    assert_eq!(
        config.validate(),
        Err(TokenConfigError::DuplicateBucket(
            AllocationBucket::EcosystemIncentives
        ))
    );
}

#[test]
fn token_config_validate_rejects_supply_mismatch() {
    let mut config = default_token_config();
    if let Some(allocation) = config.allocations.get_mut(0) {
        allocation.amount += 1;
    }

    assert_eq!(
        config.validate(),
        Err(TokenConfigError::AllocationAmountSum {
            expected: DEFAULT_TOTAL_SUPPLY,
            actual: DEFAULT_TOTAL_SUPPLY + 1,
        })
    );
}

#[test]
fn token_config_rejects_invalid_symbol() {
    let config = TokenConfig {
        symbol: "kamn".to_owned(),
        total_supply: DEFAULT_TOTAL_SUPPLY,
        decimals: DEFAULT_DECIMALS,
        allocations: default_token_config().allocations,
    };

    assert_eq!(
        config.validate(),
        Err(TokenConfigError::InvalidSymbol("kamn".to_owned()))
    );
}
