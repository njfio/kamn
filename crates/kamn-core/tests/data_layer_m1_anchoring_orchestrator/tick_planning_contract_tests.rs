use kamn_core::{
    DataLayerM1AnchorRetryClass, DataLayerM1AnchoringFollowUpAction,
    DataLayerM1AnchoringFollowUpPolicy, DataLayerM1AnchoringTickOutcome,
    KolmeCommitReceiptFinality, DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_DEFERRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE,
};

use super::support::{memory_orchestrator, pending};

#[test]
fn spec_c01_orchestrator_tick_defers_when_scheduler_thresholds_are_not_met() {
    let mut orchestrator = memory_orchestrator("kamn:did:agent:m1-orchestrator-c01", 2);

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
    let mut orchestrator = memory_orchestrator("kamn:did:agent:m1-orchestrator-c02", 1);

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
    assert_eq!(reason_code, DATA_LAYER_M1_ANCHORING_TICK_PLANNED_REASON_CODE);
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
            retry_class: DataLayerM1AnchorRetryClass::NewSubmission,
            receipt_finality: Some(KolmeCommitReceiptFinality::Pending),
        }
    );
}
