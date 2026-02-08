use kamn_core::{
    recalculate_and_persist_trust_score, AgentReputation, CapabilityVerification, DisputeRecord,
    Endorsement, ReputationStore, TrustScoreError,
};

fn sample_reputation() -> AgentReputation {
    AgentReputation {
        agent_did: "kamn:did:agent:agent-1".to_owned(),
        trust_score: 500,
        delivery_rate: 0.9,
        response_time_avg_ms: 900,
        dispute_rate: 0.1,
        tasks_completed: 120,
        tasks_failed: 10,
        tasks_delegated: 4,
        total_earned: 10_000,
        total_spent: 4_000,
        endorsements: vec![
            Endorsement {
                endorsement_id: "endorse-1".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-1".to_owned(),
                note: "consistent delivery".to_owned(),
                block_height: 10,
            },
            Endorsement {
                endorsement_id: "endorse-2".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-2".to_owned(),
                note: "strong collaboration".to_owned(),
                block_height: 11,
            },
            Endorsement {
                endorsement_id: "endorse-3".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-3".to_owned(),
                note: "accurate responses".to_owned(),
                block_height: 12,
            },
            Endorsement {
                endorsement_id: "endorse-4".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-4".to_owned(),
                note: "excellent reliability".to_owned(),
                block_height: 13,
            },
            Endorsement {
                endorsement_id: "endorse-5".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-5".to_owned(),
                note: "clear communication".to_owned(),
                block_height: 14,
            },
        ],
        disputes: vec![DisputeRecord {
            dispute_id: "dispute-1".to_owned(),
            opened_by: "kamn:did:agent:requester-9".to_owned(),
            reason: "timeout".to_owned(),
            block_height: 15,
        }],
        verified_capabilities: vec![CapabilityVerification {
            capability: "market-analysis".to_owned(),
            verifier_did: "kamn:did:agent:verifier-1".to_owned(),
            proof_ref: "ipfs://QmCapabilityProof".to_owned(),
            block_height: 16,
        }],
        last_updated_block: 16,
        score_history: vec![],
    }
}

#[test]
fn trust_score_engine_matches_prd_8_2_formula() {
    let breakdown =
        kamn_core::calculate_trust_score(&sample_reputation()).expect("calculation should succeed");

    assert_eq!(breakdown.base_score, 500);
    assert_eq!(breakdown.delivery_component, 160);
    assert_eq!(breakdown.response_component, 100);
    assert_eq!(breakdown.dispute_penalty, 15);
    assert_eq!(breakdown.volume_bonus, 12);
    assert_eq!(breakdown.endorsement_bonus, 5);
    assert_eq!(breakdown.final_score, 762);
}

#[test]
fn trust_score_engine_rejects_invalid_rate_inputs() {
    let mut reputation = sample_reputation();
    reputation.delivery_rate = 1.2;

    assert_eq!(
        kamn_core::calculate_trust_score(&reputation),
        Err(TrustScoreError::InvalidDeliveryRate(1.2))
    );
}

#[test]
fn trust_score_engine_caps_volume_and_endorsement_bonus() {
    let mut reputation = sample_reputation();
    reputation.tasks_completed = 10_000;
    reputation.endorsements = (0..150)
        .map(|index| Endorsement {
            endorsement_id: format!("endorse-{index}"),
            from_agent_did: "kamn:did:agent:endorser-x".to_owned(),
            note: "bulk endorsement".to_owned(),
            block_height: 20 + index as u64,
        })
        .collect();

    let breakdown =
        kamn_core::calculate_trust_score(&reputation).expect("calculation should succeed");
    assert_eq!(breakdown.volume_bonus, 100);
    assert_eq!(breakdown.endorsement_bonus, 50);
}

#[test]
fn trust_score_engine_is_deterministic_for_same_input() {
    let first =
        kamn_core::calculate_trust_score(&sample_reputation()).expect("calculation should succeed");
    let second =
        kamn_core::calculate_trust_score(&sample_reputation()).expect("calculation should succeed");
    assert_eq!(first, second);
}

#[test]
fn trust_score_engine_integration_persists_score_to_store() {
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-1", 10)
        .expect("registration should succeed");
    store
        .record_task_outcome(
            "kamn:did:agent:agent-1",
            kamn_core::ReputationTaskOutcome::Completed,
            Some(900),
            100,
            0,
            11,
        )
        .expect("task update should succeed");
    store
        .record_task_outcome(
            "kamn:did:agent:agent-1",
            kamn_core::ReputationTaskOutcome::Failed,
            Some(1_100),
            0,
            25,
            12,
        )
        .expect("task update should succeed");
    store
        .record_endorsement(
            "kamn:did:agent:agent-1",
            Endorsement {
                endorsement_id: "endorse-1".to_owned(),
                from_agent_did: "kamn:did:agent:endorser-1".to_owned(),
                note: "high quality".to_owned(),
                block_height: 12,
            },
        )
        .expect("endorsement should succeed");

    let persisted = recalculate_and_persist_trust_score(&mut store, "kamn:did:agent:agent-1", 13)
        .expect("recalculation should succeed");
    let state = store
        .get_agent("kamn:did:agent:agent-1")
        .expect("agent should exist");
    assert_eq!(state.trust_score, persisted.final_score);
    assert_eq!(
        state.score_history.last().map(|entry| entry.block_height),
        Some(13)
    );
}

#[test]
fn trust_score_engine_regression_response_1000_uses_fastest_bucket() {
    // Regression: #213
    let mut reputation = sample_reputation();
    reputation.response_time_avg_ms = 1_000;
    let breakdown =
        kamn_core::calculate_trust_score(&reputation).expect("calculation should succeed");
    assert_eq!(breakdown.response_component, 100);
}
