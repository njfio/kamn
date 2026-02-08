use kamn_core::{
    AgentReputation, CapabilityVerification, DisputeRecord, Endorsement, ReputationError,
    ReputationStore, ReputationTaskOutcome,
};

fn sample_endorsement(id: &str) -> Endorsement {
    Endorsement {
        endorsement_id: id.to_owned(),
        from_agent_did: "kamn:did:agent:endorser-1".to_owned(),
        note: "high quality delivery".to_owned(),
        block_height: 11,
    }
}

fn sample_dispute(id: &str) -> DisputeRecord {
    DisputeRecord {
        dispute_id: id.to_owned(),
        opened_by: "kamn:did:agent:requester-2".to_owned(),
        reason: "late artifact submission".to_owned(),
        block_height: 12,
    }
}

fn sample_capability(capability: &str) -> CapabilityVerification {
    CapabilityVerification {
        capability: capability.to_owned(),
        verifier_did: "kamn:did:agent:verifier-1".to_owned(),
        proof_ref: "ipfs://QmCapabilityProof".to_owned(),
        block_height: 13,
    }
}

fn read_agent(store: &ReputationStore, did: &str) -> AgentReputation {
    store
        .get_agent(did)
        .expect("agent should exist in reputation store")
        .clone()
}

#[test]
fn reputation_state_registers_agent_and_updates_core_metrics() {
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-1", 10)
        .expect("registration should succeed");

    store
        .record_task_outcome(
            "kamn:did:agent:agent-1",
            ReputationTaskOutcome::Completed,
            Some(1_200),
            250,
            20,
            11,
        )
        .expect("completed task update should succeed");
    store
        .record_task_outcome(
            "kamn:did:agent:agent-1",
            ReputationTaskOutcome::Failed,
            Some(4_000),
            0,
            12,
            12,
        )
        .expect("failed task update should succeed");
    store
        .record_task_outcome(
            "kamn:did:agent:agent-1",
            ReputationTaskOutcome::Delegated,
            None,
            0,
            0,
            13,
        )
        .expect("delegated task update should succeed");

    let agent = read_agent(&store, "kamn:did:agent:agent-1");
    assert_eq!(agent.tasks_completed, 1);
    assert_eq!(agent.tasks_failed, 1);
    assert_eq!(agent.tasks_delegated, 1);
    assert_eq!(agent.total_earned, 250);
    assert_eq!(agent.total_spent, 32);
    assert_eq!(agent.delivery_rate, 0.5);
    assert_eq!(agent.response_time_avg_ms, 2_600);
}

#[test]
fn reputation_state_persists_attestations_and_capabilities() {
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-2", 20)
        .expect("registration should succeed");

    store
        .record_endorsement("kamn:did:agent:agent-2", sample_endorsement("endorse-1"))
        .expect("endorsement should succeed");
    store
        .record_dispute("kamn:did:agent:agent-2", sample_dispute("dispute-1"))
        .expect("dispute should succeed");
    store
        .record_capability_verification(
            "kamn:did:agent:agent-2",
            sample_capability("market-analysis"),
        )
        .expect("capability verification should succeed");

    let agent = read_agent(&store, "kamn:did:agent:agent-2");
    assert_eq!(agent.endorsements.len(), 1);
    assert_eq!(agent.disputes.len(), 1);
    assert_eq!(agent.verified_capabilities.len(), 1);
    assert_eq!(agent.dispute_rate, 1.0);
}

#[test]
fn reputation_state_exports_and_restores_deterministic_records() {
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-1", 5)
        .expect("registration should succeed");
    store
        .register_agent("kamn:did:agent:agent-2", 6)
        .expect("registration should succeed");
    store
        .set_trust_score("kamn:did:agent:agent-1", 620, 7)
        .expect("score update should succeed");

    let exported = store.export_records();
    assert_eq!(exported.len(), 2);
    assert_eq!(
        exported[0].state_key,
        "kamn.reputation.scores:agent:agent-1".to_owned()
    );
    assert_eq!(
        exported[1].state_key,
        "kamn.reputation.scores:agent:agent-2".to_owned()
    );

    let restored = ReputationStore::restore_from_records(&exported)
        .expect("restore from persisted records should succeed");
    let restored_agent = read_agent(&restored, "kamn:did:agent:agent-1");
    assert_eq!(restored_agent.trust_score, 620);
    assert_eq!(restored_agent.score_history.len(), 2);
}

#[test]
fn reputation_state_rejects_invalid_agent_did() {
    let mut store = ReputationStore::default();
    assert_eq!(
        store.register_agent("did:example:agent-1", 1),
        Err(ReputationError::InvalidAgentDid(
            "invalid agent did prefix: did:example:agent-1".to_owned()
        ))
    );
}

#[test]
fn reputation_state_rejects_duplicate_endorsement_id() {
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-4", 15)
        .expect("registration should succeed");
    store
        .record_endorsement("kamn:did:agent:agent-4", sample_endorsement("endorse-1"))
        .expect("first endorsement should succeed");

    assert_eq!(
        store.record_endorsement("kamn:did:agent:agent-4", sample_endorsement("endorse-1")),
        Err(ReputationError::DuplicateEndorsementId(
            "endorse-1".to_owned()
        ))
    );
}

#[test]
fn reputation_state_regression_accepts_upper_bound_trust_score() {
    // Regression: #215
    let mut store = ReputationStore::default();
    store
        .register_agent("kamn:did:agent:agent-5", 30)
        .expect("registration should succeed");

    store
        .set_trust_score("kamn:did:agent:agent-5", 1_000, 31)
        .expect("upper bound score should be accepted");
    let agent = read_agent(&store, "kamn:did:agent:agent-5");
    assert_eq!(agent.trust_score, 1_000);
}
