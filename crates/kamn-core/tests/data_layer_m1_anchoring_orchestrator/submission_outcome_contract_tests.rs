use kamn_core::{
    DataLayerM1AnchoringConfirmationMetadata, DataLayerM1AnchoringFollowUpAction,
    DataLayerM1AnchoringOrchestratorError, DataLayerM1AnchoringTickOutcome,
    KolmeCommitReceiptFinality, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitReceipt,
    DATA_LAYER_M1_ANCHORING_CONFIRMATION_HINT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_CONFLICT_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_RETRY_IN_FLIGHT_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_REJECTED_REASON_CODE,
};

use super::support::{pending, scripted_orchestrator};

#[test]
fn spec_c04_final_receipt_requires_confirmation_metadata() {
    assert_missing_confirmation_error();
    assert_final_confirmation_projection();
}

#[test]
fn spec_c04_rejected_anchor_projects_rejected_outcome() {
    let mut orchestrator = scripted_orchestrator(
        "kamn:did:agent:m1-orchestrator-c04-rejected",
        vec![KolmeRuntimeCommitOutcome::Rejected {
            reason: "provider-policy-rejection".to_owned(),
        }],
    );

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

    assert_rejected_outcome(outcome);
}

#[test]
fn spec_c04_duplicate_pending_anchor_projects_retry_follow_up_policy() {
    let outcome = duplicate_pending_outcome();
    assert_duplicate_retry_follow_up(outcome);
}

fn final_receipt(commit_id: &str) -> KolmeRuntimeCommitOutcome {
    KolmeRuntimeCommitOutcome::Submitted(KolmeRuntimeCommitReceipt {
        provider: "kolme-scripted".to_owned(),
        commit_id: commit_id.to_owned(),
        finality: KolmeCommitReceiptFinality::Final,
    })
}

fn assert_missing_confirmation_error() {
    let mut orchestrator = scripted_orchestrator(
        "kamn:did:agent:m1-orchestrator-c04",
        vec![final_receipt("commit-c04-final")],
    );
    let missing_confirmation = plan_scripted_tick(
        &mut orchestrator,
        "00000000-0000-0000-0000-000000000401",
        "sha256:c04a",
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
}

fn assert_final_confirmation_projection() {
    let confirmation = DataLayerM1AnchoringConfirmationMetadata {
        kolme_block_height: 123_456,
        confirmed_at_unix_seconds: 1_900_000_020,
    };
    let mut orchestrator = scripted_orchestrator(
        "kamn:did:agent:m1-orchestrator-c04-ok",
        vec![final_receipt("commit-c04-final-ok")],
    );
    let outcome = plan_scripted_tick(
        &mut orchestrator,
        "00000000-0000-0000-0000-000000000402",
        "sha256:c04b",
        Some(confirmation.clone()),
    )
    .expect("final receipt with confirmation metadata should succeed");
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    assert_eq!(persistence_plan.confirmation, Some(confirmation));
}

fn assert_rejected_outcome(outcome: DataLayerM1AnchoringTickOutcome) {
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

fn assert_duplicate_retry_follow_up(outcome: DataLayerM1AnchoringTickOutcome) {
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
        follow_up_policy.receipt_finality,
        Some(KolmeCommitReceiptFinality::Pending)
    );
}

fn duplicate_pending_outcome() -> DataLayerM1AnchoringTickOutcome {
    let mut orchestrator = scripted_orchestrator(
        "kamn:did:agent:m1-orchestrator-c04-duplicate",
        vec![KolmeRuntimeCommitOutcome::Duplicate(
            KolmeRuntimeCommitReceipt {
                provider: "kolme-scripted".to_owned(),
                commit_id: "commit-c04-duplicate".to_owned(),
                finality: KolmeCommitReceiptFinality::Pending,
            },
        )],
    );
    plan_scripted_tick(
        &mut orchestrator,
        "00000000-0000-0000-0000-000000000404",
        "sha256:c04d",
        None,
    )
    .expect("duplicate pending anchors should evaluate deterministically")
}

fn plan_scripted_tick(
    orchestrator: &mut kamn_core::DataLayerM1AnchoringOrchestrator<
        super::support::ScriptedKolmeRuntimeCommitClient,
    >,
    message_id: &str,
    content_hash: &str,
    confirmation: Option<DataLayerM1AnchoringConfirmationMetadata>,
) -> Result<DataLayerM1AnchoringTickOutcome, DataLayerM1AnchoringOrchestratorError> {
    orchestrator.plan_tick(
        &[pending(message_id, content_hash, 1_900_000_000)],
        1_900_000_010,
        1_900_000_010,
        1_900_000_011,
        confirmation,
    )
}
