use kamn_core::{
    DataLayerM1AnchoringConfirmationMetadata, DataLayerM1AnchoringOrchestrator,
    DataLayerM1AnchoringOrchestratorError, DataLayerM1AnchoringTickOutcome,
    DataLayerM1BatchSchedulerPolicy, DataLayerM1PendingBatchMessage,
    InMemoryKolmeRuntimeCommitClient, KolmeCommitReceiptFinality, KolmeRuntimeCommitClient,
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt,
    KolmeRuntimeCommitRequest, DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE,
};
use std::collections::VecDeque;

fn pending(
    message_id: &str,
    content_hash: &str,
    created_at_unix_seconds: u64,
) -> DataLayerM1PendingBatchMessage {
    DataLayerM1PendingBatchMessage {
        message_id: message_id.to_owned(),
        content_hash: content_hash.to_owned(),
        created_at_unix_seconds,
    }
}

#[derive(Debug, Clone)]
struct ScriptedKolmeRuntimeCommitClient {
    scripted_outcomes: VecDeque<KolmeRuntimeCommitOutcome>,
}

impl ScriptedKolmeRuntimeCommitClient {
    fn new(scripted_outcomes: Vec<KolmeRuntimeCommitOutcome>) -> Self {
        Self {
            scripted_outcomes: scripted_outcomes.into(),
        }
    }
}

impl KolmeRuntimeCommitClient for ScriptedKolmeRuntimeCommitClient {
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;
        Ok(self
            .scripted_outcomes
            .pop_front()
            .unwrap_or(KolmeRuntimeCommitOutcome::Rejected {
                reason: "scripted_outcome_exhausted".to_owned(),
            }))
    }
}

#[test]
fn spec_c01_orchestrator_tick_defers_when_scheduler_thresholds_are_not_met() {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy = DataLayerM1BatchSchedulerPolicy::new(2, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c01",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending("msg-c01-a", "sha256:c01a", 1_900_000_000)],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("tick should evaluate");
    assert_eq!(
        outcome,
        DataLayerM1AnchoringTickOutcome::Deferred {
            reason_code: DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE,
            pending_count: 1,
        }
    );
}

#[test]
fn spec_c02_orchestrator_tick_projects_planned_persistence_metadata() {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c02",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000201",
                "sha256:c02a",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("tick should evaluate");

    let DataLayerM1AnchoringTickOutcome::Planned {
        reason_code,
        persistence_plan,
        ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    assert_eq!(
        reason_code,
        DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE
    );
    assert_eq!(persistence_plan.leaf_count, 1);
    assert_eq!(persistence_plan.assignments.len(), 1);
    assert_eq!(
        persistence_plan.assignments[0].message_id,
        "00000000-0000-0000-0000-000000000201"
    );
    assert!(persistence_plan.submission.is_some());
    assert!(persistence_plan.confirmation.is_none());
}

#[test]
fn spec_c04_final_receipt_requires_confirmation_metadata() {
    let client = ScriptedKolmeRuntimeCommitClient::new(vec![KolmeRuntimeCommitOutcome::Submitted(
        KolmeRuntimeCommitReceipt {
            provider: "kolme-scripted".to_owned(),
            commit_id: "commit-c04-final".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        },
    )]);
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c04",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let missing_confirmation = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000401",
                "sha256:c04a",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect_err("missing confirmation metadata should fail closed");
    assert_eq!(
        missing_confirmation,
        DataLayerM1AnchoringOrchestratorError::MissingConfirmationMetadata {
            reason_code: DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
            transaction_id: "commit-c04-final".to_owned(),
        }
    );

    let client = ScriptedKolmeRuntimeCommitClient::new(vec![KolmeRuntimeCommitOutcome::Submitted(
        KolmeRuntimeCommitReceipt {
            provider: "kolme-scripted".to_owned(),
            commit_id: "commit-c04-final-ok".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        },
    )]);
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c04-ok",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000402",
                "sha256:c04b",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            Some(DataLayerM1AnchoringConfirmationMetadata {
                kolme_block_height: 123_456,
                confirmed_at_unix_seconds: 1_900_000_020,
            }),
        )
        .expect("final receipt with confirmation metadata should succeed");
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    assert_eq!(
        persistence_plan.confirmation,
        Some(DataLayerM1AnchoringConfirmationMetadata {
            kolme_block_height: 123_456,
            confirmed_at_unix_seconds: 1_900_000_020,
        })
    );
}

#[test]
fn spec_c04_rejected_anchor_projects_rejected_outcome() {
    let client = ScriptedKolmeRuntimeCommitClient::new(vec![KolmeRuntimeCommitOutcome::Rejected {
        reason: "provider-policy-rejection".to_owned(),
    }]);
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c04-rejected",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000403",
                "sha256:c04c",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("rejected anchors should project deterministic outcome");

    let DataLayerM1AnchoringTickOutcome::Rejected {
        reason_code,
        rejection_reason,
        ..
    } = outcome
    else {
        panic!("expected rejected outcome");
    };
    assert_eq!(
        reason_code,
        DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE
    );
    assert_eq!(rejection_reason, "provider-policy-rejection".to_owned());
}
