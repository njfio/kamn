use kamn_core::{
    rank_agents_for_routing, rank_listings_by_reputation, CapabilityVerification, Endorsement,
    ReputationSignalError, ReputationStore, RoutingSignalWeights, ServiceListing,
};

fn register_agent(store: &mut ReputationStore, did: &str, trust_score: u32) {
    store
        .register_agent(did, 10)
        .expect("registration should succeed");
    store
        .set_trust_score(did, trust_score, 11)
        .expect("score should update");
}

fn add_endorsement(store: &mut ReputationStore, did: &str, id: &str) {
    store
        .record_endorsement(
            did,
            Endorsement {
                endorsement_id: id.to_owned(),
                from_agent_did: "kamn:did:agent:endorser".to_owned(),
                note: "trusted partner".to_owned(),
                block_height: 12,
            },
        )
        .expect("endorsement should succeed");
}

fn add_capability(store: &mut ReputationStore, did: &str, capability: &str) {
    store
        .record_capability_verification(
            did,
            CapabilityVerification {
                capability: capability.to_owned(),
                verifier_did: "kamn:did:agent:verifier".to_owned(),
                proof_ref: "ipfs://QmCapabilityProof".to_owned(),
                block_height: 13,
            },
        )
        .expect("capability verification should succeed");
}

fn sample_listing(id: &str, provider_did: &str) -> ServiceListing {
    ServiceListing {
        listing_id: id.to_owned(),
        provider_did: provider_did.to_owned(),
        service_name: "analysis".to_owned(),
        category: "research".to_owned(),
        tags: vec!["market-analysis".to_owned()],
        hourly_rate: 100,
        negotiation_channel_id: "channel-marketplace".to_owned(),
    }
}

#[test]
fn reputation_signal_routing_ranks_agents_using_signals_and_capabilities() {
    let mut store = ReputationStore::default();
    register_agent(&mut store, "kamn:did:agent:agent-a", 700);
    register_agent(&mut store, "kamn:did:agent:agent-b", 720);
    register_agent(&mut store, "kamn:did:agent:agent-c", 680);

    add_endorsement(&mut store, "kamn:did:agent:agent-a", "endorse-a1");
    add_endorsement(&mut store, "kamn:did:agent:agent-a", "endorse-a2");
    add_capability(&mut store, "kamn:did:agent:agent-a", "market-analysis");

    store
        .record_dispute(
            "kamn:did:agent:agent-b",
            kamn_core::DisputeRecord {
                dispute_id: "dispute-b1".to_owned(),
                opened_by: "kamn:did:agent:requester-1".to_owned(),
                reason: "late response".to_owned(),
                block_height: 14,
            },
        )
        .expect("dispute should succeed");

    add_capability(&mut store, "kamn:did:agent:agent-c", "market-analysis");

    let ranked = rank_agents_for_routing(
        &store,
        &[
            "kamn:did:agent:agent-a",
            "kamn:did:agent:agent-b",
            "kamn:did:agent:agent-c",
        ],
        &["market-analysis"],
        RoutingSignalWeights::default(),
    )
    .expect("ranking should succeed");

    assert_eq!(ranked.len(), 3);
    assert_eq!(ranked[0].agent_did, "kamn:did:agent:agent-a");
    assert!(ranked[1].routing_score >= ranked[2].routing_score);
    let disputed = ranked
        .iter()
        .find(|candidate| candidate.agent_did == "kamn:did:agent:agent-b")
        .expect("agent-b result should exist");
    assert!(disputed.signal_adjustment < 0);
}

#[test]
fn reputation_signal_routing_integration_ranks_marketplace_listings() {
    let mut store = ReputationStore::default();
    register_agent(&mut store, "kamn:did:agent:provider-a", 650);
    register_agent(&mut store, "kamn:did:agent:provider-b", 650);
    add_endorsement(&mut store, "kamn:did:agent:provider-a", "endorse-a1");
    add_capability(&mut store, "kamn:did:agent:provider-a", "market-analysis");

    let ranked = rank_listings_by_reputation(
        &[
            sample_listing("listing-a", "kamn:did:agent:provider-a"),
            sample_listing("listing-b", "kamn:did:agent:provider-b"),
        ],
        &store,
        &["market-analysis"],
        RoutingSignalWeights::default(),
    )
    .expect("listing ranking should succeed");

    assert_eq!(ranked[0].listing_id, "listing-a");
    assert_eq!(ranked[0].provider_did, "kamn:did:agent:provider-a");
}

#[test]
fn reputation_signal_routing_rejects_invalid_candidate_did() {
    let store = ReputationStore::default();
    let result = rank_agents_for_routing(
        &store,
        &["did:example:agent-1"],
        &[],
        RoutingSignalWeights::default(),
    );
    assert_eq!(
        result,
        Err(ReputationSignalError::InvalidCandidateDid(
            "invalid agent did prefix: did:example:agent-1".to_owned()
        ))
    );
}

#[test]
fn reputation_signal_routing_regression_uses_did_tiebreak_for_equal_scores() {
    // Regression: #211
    let mut store = ReputationStore::default();
    register_agent(&mut store, "kamn:did:agent:agent-1", 700);
    register_agent(&mut store, "kamn:did:agent:agent-2", 700);

    let ranked = rank_agents_for_routing(
        &store,
        &["kamn:did:agent:agent-2", "kamn:did:agent:agent-1"],
        &[],
        RoutingSignalWeights::default(),
    )
    .expect("ranking should succeed");

    assert_eq!(ranked[0].agent_did, "kamn:did:agent:agent-1");
    assert_eq!(ranked[1].agent_did, "kamn:did:agent:agent-2");
}
