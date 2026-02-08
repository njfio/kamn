use kamn_core::{
    ChannelStore, ChannelType, MarketplaceSearchFilter, NegotiationThreadHook, ServiceListing,
    ServiceMarketplaceEngine, ServiceMarketplaceError,
};

fn marketplace_channels() -> ChannelStore {
    let mut channels = ChannelStore::new();
    channels
        .create_marketplace_channel(
            "chan-market-1",
            "kamn:did:agent:provider-1",
            "service-market-v1",
            vec![
                "kamn:did:agent:provider-1".to_owned(),
                "kamn:did:agent:requester-1".to_owned(),
            ],
            vec!["kamn:did:agent:provider-1".to_owned()],
        )
        .expect("marketplace channel should be created");
    channels
}

#[test]
fn listing_rejects_non_marketplace_channel_type() {
    let mut channels = ChannelStore::new();
    channels
        .create_task_channel(
            "chan-task-1",
            "kamn:did:agent:provider-1",
            "task-1",
            vec![
                "kamn:did:agent:provider-1".to_owned(),
                "kamn:did:agent:requester-1".to_owned(),
            ],
            vec!["kamn:did:agent:provider-1".to_owned()],
        )
        .expect("task channel should be created");
    let mut engine = ServiceMarketplaceEngine::new();

    assert_eq!(
        engine.register_listing(
            ServiceListing {
                listing_id: "listing-1".to_owned(),
                provider_did: "kamn:did:agent:provider-1".to_owned(),
                service_name: "Smart Contract Audit".to_owned(),
                category: "security".to_owned(),
                tags: vec!["solidity".to_owned(), "formal-review".to_owned()],
                hourly_rate: 150,
                negotiation_channel_id: "chan-task-1".to_owned(),
            },
            &channels,
        ),
        Err(ServiceMarketplaceError::NegotiationChannelType {
            channel_id: "chan-task-1".to_owned(),
            found: ChannelType::Task,
        })
    );
}

#[test]
fn search_filters_are_deterministic() {
    let channels = marketplace_channels();
    let mut engine = ServiceMarketplaceEngine::new();

    engine
        .register_listing(
            ServiceListing {
                listing_id: "listing-1".to_owned(),
                provider_did: "kamn:did:agent:provider-1".to_owned(),
                service_name: "Rust Protocol Review".to_owned(),
                category: "security".to_owned(),
                tags: vec!["rust".to_owned(), "protocol".to_owned()],
                hourly_rate: 120,
                negotiation_channel_id: "chan-market-1".to_owned(),
            },
            &channels,
        )
        .expect("first listing should register");
    engine
        .register_listing(
            ServiceListing {
                listing_id: "listing-2".to_owned(),
                provider_did: "kamn:did:agent:provider-1".to_owned(),
                service_name: "Go Reliability Review".to_owned(),
                category: "operations".to_owned(),
                tags: vec!["go".to_owned(), "sre".to_owned()],
                hourly_rate: 90,
                negotiation_channel_id: "chan-market-1".to_owned(),
            },
            &channels,
        )
        .expect("second listing should register");

    let filtered = engine.search(&MarketplaceSearchFilter {
        category: Some("security".to_owned()),
        tag: Some("rust".to_owned()),
        max_hourly_rate: Some(150),
    });
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].listing_id, "listing-1");
}

#[test]
fn negotiation_thread_hook_maps_listing_to_channel() {
    let channels = marketplace_channels();
    let mut engine = ServiceMarketplaceEngine::new();
    engine
        .register_listing(
            ServiceListing {
                listing_id: "listing-3".to_owned(),
                provider_did: "kamn:did:agent:provider-1".to_owned(),
                service_name: "Incident Retrospective Facilitation".to_owned(),
                category: "operations".to_owned(),
                tags: vec!["incident".to_owned()],
                hourly_rate: 110,
                negotiation_channel_id: "chan-market-1".to_owned(),
            },
            &channels,
        )
        .expect("listing should register");

    assert_eq!(
        engine.open_negotiation_thread("listing-3", "kamn:did:agent:requester-9"),
        Ok(NegotiationThreadHook {
            listing_id: "listing-3".to_owned(),
            negotiation_channel_id: "chan-market-1".to_owned(),
            provider_did: "kamn:did:agent:provider-1".to_owned(),
            requester_did: "kamn:did:agent:requester-9".to_owned(),
        })
    );
}

#[test]
fn regression_provider_must_be_marketplace_channel_member() {
    // Regression: #188
    let mut channels = ChannelStore::new();
    channels
        .create_marketplace_channel(
            "chan-market-2",
            "kamn:did:agent:owner-1",
            "service-market-v1",
            vec![
                "kamn:did:agent:owner-1".to_owned(),
                "kamn:did:agent:requester-1".to_owned(),
            ],
            vec!["kamn:did:agent:owner-1".to_owned()],
        )
        .expect("marketplace channel should be created");

    let mut engine = ServiceMarketplaceEngine::new();
    assert_eq!(
        engine.register_listing(
            ServiceListing {
                listing_id: "listing-4".to_owned(),
                provider_did: "kamn:did:agent:provider-x".to_owned(),
                service_name: "Threat Modeling".to_owned(),
                category: "security".to_owned(),
                tags: vec!["threat-model".to_owned()],
                hourly_rate: 130,
                negotiation_channel_id: "chan-market-2".to_owned(),
            },
            &channels,
        ),
        Err(ServiceMarketplaceError::ProviderNotChannelMember {
            provider_did: "kamn:did:agent:provider-x".to_owned(),
            channel_id: "chan-market-2".to_owned(),
        })
    );
}
