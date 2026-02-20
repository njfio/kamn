use kamn_core::{
    reconcile_data_layer_m1_finality_observation, DataLayerM1AnchoringConfirmationMetadata,
    DataLayerM1AnchoringFinalityObservation, DataLayerM1AnchoringFollowUpAction,
    DataLayerM1AnchoringFollowUpPolicy, DataLayerM1AnchoringOrchestrator,
    DataLayerM1AnchoringOrchestratorError, DataLayerM1AnchoringTickOutcome,
    DataLayerM1BatchSchedulerPolicy, DataLayerM1PendingBatchMessage,
    InMemoryKolmeRuntimeCommitClient, KolmeCommitReceiptFinality, KolmeRuntimeCommitClient,
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt,
    KolmeRuntimeCommitRequest, DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_FINAL_BLOCK_HEIGHT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_TX_MISMATCH_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE,
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
        follow_up_policy,
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
    assert_eq!(
        follow_up_policy,
        DataLayerM1AnchoringFollowUpPolicy {
            action: DataLayerM1AnchoringFollowUpAction::PollConfirmation,
            reason_code: DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
            retry_after_unix_seconds: None,
            poll_after_unix_seconds: Some(1_900_000_040),
            retry_class: kamn_core::DataLayerM1AnchorRetryClass::NewSubmission,
            receipt_finality: Some(KolmeCommitReceiptFinality::Pending),
        }
    );
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
        follow_up_policy,
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
    assert_eq!(
        follow_up_policy.action,
        DataLayerM1AnchoringFollowUpAction::NoRetry
    );
    assert_eq!(
        follow_up_policy.reason_code,
        DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE
    );
    assert!(follow_up_policy.retry_after_unix_seconds.is_none());
    assert!(follow_up_policy.poll_after_unix_seconds.is_none());
}

#[test]
fn spec_c04_duplicate_pending_anchor_projects_retry_follow_up_policy() {
    let client = ScriptedKolmeRuntimeCommitClient::new(vec![KolmeRuntimeCommitOutcome::Duplicate(
        KolmeRuntimeCommitReceipt {
            provider: "kolme-scripted".to_owned(),
            commit_id: "commit-c04-duplicate".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
        },
    )]);
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c04-duplicate",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000404",
                "sha256:c04d",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("duplicate pending anchors should evaluate deterministically");

    let DataLayerM1AnchoringTickOutcome::Planned {
        follow_up_policy, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    assert_eq!(
        follow_up_policy.action,
        DataLayerM1AnchoringFollowUpAction::Retry
    );
    assert_eq!(
        follow_up_policy.reason_code,
        DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE
    );
    assert_eq!(
        follow_up_policy.retry_after_unix_seconds,
        Some(1_900_000_070)
    );
    assert!(follow_up_policy.poll_after_unix_seconds.is_none());
    assert_eq!(
        follow_up_policy.retry_class,
        kamn_core::DataLayerM1AnchorRetryClass::RetryableInFlight
    );
    assert_eq!(
        follow_up_policy.receipt_finality,
        Some(KolmeCommitReceiptFinality::Pending)
    );
}

#[test]
fn spec_c03_reconcile_pending_and_final_finality_observation_projects_deterministic_updates() {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c03-reconcile",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000501",
                "sha256:c03a",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("planned outcome should evaluate");
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = &outcome
    else {
        panic!("expected planned outcome");
    };
    let submission = persistence_plan
        .submission
        .as_ref()
        .expect("planned outcome should include submission metadata");

    let pending_projection = reconcile_data_layer_m1_finality_observation(
        &outcome,
        &DataLayerM1AnchoringFinalityObservation {
            provider: "kolme-memory".to_owned(),
            transaction_id: submission.kolme_tx_hash.clone(),
            finality: KolmeCommitReceiptFinality::Pending,
            block_height: None,
            observed_at_unix_seconds: 1_900_000_050,
        },
    )
    .expect("pending finality reconciliation should succeed");
    assert_eq!(
        pending_projection.follow_up_policy.action,
        DataLayerM1AnchoringFollowUpAction::PollConfirmation
    );
    assert_eq!(pending_projection.confirmation, None);

    let final_projection = reconcile_data_layer_m1_finality_observation(
        &outcome,
        &DataLayerM1AnchoringFinalityObservation {
            provider: "kolme-memory".to_owned(),
            transaction_id: submission.kolme_tx_hash.clone(),
            finality: KolmeCommitReceiptFinality::Final,
            block_height: Some(123_456),
            observed_at_unix_seconds: 1_900_000_090,
        },
    )
    .expect("final finality reconciliation should succeed");
    assert_eq!(
        final_projection.follow_up_policy.reason_code,
        DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE
    );
    assert_eq!(
        final_projection.confirmation,
        Some(DataLayerM1AnchoringConfirmationMetadata {
            kolme_block_height: 123_456,
            confirmed_at_unix_seconds: 1_900_000_090,
        })
    );
}

#[test]
fn spec_c04_reconcile_finality_observation_fails_closed_for_mismatch_and_missing_block_height() {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    let mut orchestrator = DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:m1-orchestrator-c04-finality-fail",
        "m1-root",
        policy,
    )
    .expect("orchestrator should initialize");

    let outcome = orchestrator
        .plan_tick(
            &[pending(
                "00000000-0000-0000-0000-000000000601",
                "sha256:c04e",
                1_900_000_000,
            )],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("planned outcome should evaluate");
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = &outcome
    else {
        panic!("expected planned outcome");
    };
    let submission = persistence_plan
        .submission
        .as_ref()
        .expect("planned outcome should include submission metadata");

    let mismatch = reconcile_data_layer_m1_finality_observation(
        &outcome,
        &DataLayerM1AnchoringFinalityObservation {
            provider: "kolme-memory".to_owned(),
            transaction_id: "tx-unexpected".to_owned(),
            finality: KolmeCommitReceiptFinality::Pending,
            block_height: None,
            observed_at_unix_seconds: 1_900_000_060,
        },
    )
    .expect_err("mismatched tx hash should fail closed");
    assert_eq!(
        mismatch,
        DataLayerM1AnchoringOrchestratorError::FinalityObservationTxMismatch {
            reason_code: DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_TX_MISMATCH_REASON_CODE,
            expected_transaction_id: submission.kolme_tx_hash.clone(),
            observed_transaction_id: "tx-unexpected".to_owned(),
        }
    );

    let missing_block_height = reconcile_data_layer_m1_finality_observation(
        &outcome,
        &DataLayerM1AnchoringFinalityObservation {
            provider: "kolme-memory".to_owned(),
            transaction_id: submission.kolme_tx_hash.clone(),
            finality: KolmeCommitReceiptFinality::Final,
            block_height: None,
            observed_at_unix_seconds: 1_900_000_070,
        },
    )
    .expect_err("final observation without block height should fail closed");
    assert_eq!(
        missing_block_height,
        DataLayerM1AnchoringOrchestratorError::MissingFinalityObservationBlockHeight {
            reason_code:
                DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_FINAL_BLOCK_HEIGHT_REQUIRED_REASON_CODE,
            transaction_id: submission.kolme_tx_hash.clone(),
        }
    );
}
