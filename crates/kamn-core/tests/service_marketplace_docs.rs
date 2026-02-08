const DOC: &str = include_str!("../../../docs/foundation/service-marketplace-discovery.md");

#[test]
fn doc_contains_listing_contract_and_engine_surfaces() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("ServiceListing"));
    assert!(DOC.contains("MarketplaceSearchFilter"));
    assert!(DOC.contains("ServiceMarketplaceEngine"));
    assert!(DOC.contains("NegotiationThreadHook"));
}

#[test]
fn doc_contains_channel_membership_and_filter_rules() {
    assert!(DOC.contains("## Listing Validation Rules"));
    assert!(DOC.contains("negotiation channel type of `Marketplace`."));
    assert!(DOC.contains("provider DID membership in the negotiation channel."));
    assert!(DOC.contains("## Discovery and Negotiation Rules"));
    assert!(DOC.contains("exact tag membership."));
}

#[test]
fn regression_requires_provider_membership_validation_rule() {
    // Regression: #188
    assert!(DOC.contains("provider DID membership in the negotiation channel."));
}
