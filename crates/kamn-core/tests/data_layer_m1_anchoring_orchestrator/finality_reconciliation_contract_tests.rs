use kamn_core::{
    reconcile_data_layer_m1_finality_observation, DataLayerM1AnchoringConfirmationMetadata,
    DataLayerM1AnchoringFinalityObservation, DataLayerM1AnchoringFollowUpAction,
    DataLayerM1AnchoringOrchestratorError, DataLayerM1AnchoringTickOutcome,
    KolmeCommitReceiptFinality,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_FINAL_BLOCK_HEIGHT_REQUIRED_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_TX_MISMATCH_REASON_CODE,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_NO_RETRY_FINAL_REASON_CODE,
};

use super::support::{memory_orchestrator, pending};

#[test]
fn spec_c03_reconcile_pending_and_final_finality_observation_projects_deterministic_updates() {
    let mut orchestrator = memory_orchestrator("kamn:did:agent:m1-orchestrator-c03-reconcile", 1);
    let outcome = planned_outcome(
        &mut orchestrator,
        "00000000-0000-0000-0000-000000000501",
        "sha256:c03a",
    );
    let submission = submission_metadata(&outcome);

    assert_pending_projection(&outcome, submission.kolme_tx_hash.as_str());
    assert_final_projection(&outcome, submission.kolme_tx_hash.as_str());
}

#[test]
fn spec_c04_reconcile_finality_observation_fails_closed_for_mismatch_and_missing_block_height() {
    let mut orchestrator =
        memory_orchestrator("kamn:did:agent:m1-orchestrator-c04-finality-fail", 1);
    let outcome = planned_outcome(
        &mut orchestrator,
        "00000000-0000-0000-0000-000000000601",
        "sha256:c04e",
    );
    let submission = submission_metadata(&outcome);

    assert_mismatch_error(&outcome, submission.kolme_tx_hash.as_str());
    assert_missing_block_height_error(&outcome, submission.kolme_tx_hash.as_str());
}

fn planned_outcome<C: kamn_core::KolmeRuntimeCommitClient>(
    orchestrator: &mut kamn_core::DataLayerM1AnchoringOrchestrator<C>,
    message_id: &str,
    content_hash: &str,
) -> DataLayerM1AnchoringTickOutcome {
    orchestrator
        .plan_tick(
            &[pending(message_id, content_hash, 1_900_000_000)],
            1_900_000_010,
            1_900_000_010,
            1_900_000_011,
            None,
        )
        .expect("planned outcome should evaluate")
}

fn submission_metadata(
    outcome: &DataLayerM1AnchoringTickOutcome,
) -> &kamn_core::DataLayerM1AnchoringSubmissionMetadata {
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    persistence_plan
        .submission
        .as_ref()
        .expect("planned outcome should include submission metadata")
}

fn assert_pending_projection(outcome: &DataLayerM1AnchoringTickOutcome, transaction_id: &str) {
    let pending_projection =
        reconcile_data_layer_m1_finality_observation(outcome, &pending_observation(transaction_id))
            .expect("pending finality reconciliation should succeed");
    assert_eq!(
        pending_projection.follow_up_policy.action,
        DataLayerM1AnchoringFollowUpAction::PollConfirmation
    );
    assert_eq!(pending_projection.confirmation, None);
}

fn assert_final_projection(outcome: &DataLayerM1AnchoringTickOutcome, transaction_id: &str) {
    let final_projection =
        reconcile_data_layer_m1_finality_observation(outcome, &final_observation(transaction_id))
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

fn assert_mismatch_error(outcome: &DataLayerM1AnchoringTickOutcome, transaction_id: &str) {
    let mismatch = reconcile_data_layer_m1_finality_observation(outcome, &mismatch_observation())
        .expect_err("mismatched tx hash should fail closed");
    assert_eq!(
        mismatch,
        DataLayerM1AnchoringOrchestratorError::FinalityObservationTxMismatch {
            reason_code: DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_TX_MISMATCH_REASON_CODE,
            expected_transaction_id: transaction_id.to_owned(),
            observed_transaction_id: "tx-unexpected".to_owned(),
        }
    );
}

fn assert_missing_block_height_error(
    outcome: &DataLayerM1AnchoringTickOutcome,
    transaction_id: &str,
) {
    let missing_block_height = reconcile_data_layer_m1_finality_observation(
        outcome,
        &missing_block_height_observation(transaction_id),
    )
    .expect_err("final observation without block height should fail closed");
    assert_eq!(
        missing_block_height,
        DataLayerM1AnchoringOrchestratorError::MissingFinalityObservationBlockHeight {
            reason_code:
                DATA_LAYER_M1_ANCHORING_FINALITY_OBSERVATION_FINAL_BLOCK_HEIGHT_REQUIRED_REASON_CODE,
            transaction_id: transaction_id.to_owned(),
        }
    );
}

fn pending_observation(transaction_id: &str) -> DataLayerM1AnchoringFinalityObservation {
    DataLayerM1AnchoringFinalityObservation {
        provider: "kolme-memory".to_owned(),
        transaction_id: transaction_id.to_owned(),
        finality: KolmeCommitReceiptFinality::Pending,
        block_height: None,
        observed_at_unix_seconds: 1_900_000_050,
    }
}

fn final_observation(transaction_id: &str) -> DataLayerM1AnchoringFinalityObservation {
    DataLayerM1AnchoringFinalityObservation {
        provider: "kolme-memory".to_owned(),
        transaction_id: transaction_id.to_owned(),
        finality: KolmeCommitReceiptFinality::Final,
        block_height: Some(123_456),
        observed_at_unix_seconds: 1_900_000_090,
    }
}

fn mismatch_observation() -> DataLayerM1AnchoringFinalityObservation {
    DataLayerM1AnchoringFinalityObservation {
        provider: "kolme-memory".to_owned(),
        transaction_id: "tx-unexpected".to_owned(),
        finality: KolmeCommitReceiptFinality::Pending,
        block_height: None,
        observed_at_unix_seconds: 1_900_000_060,
    }
}

fn missing_block_height_observation(
    transaction_id: &str,
) -> DataLayerM1AnchoringFinalityObservation {
    DataLayerM1AnchoringFinalityObservation {
        provider: "kolme-memory".to_owned(),
        transaction_id: transaction_id.to_owned(),
        finality: KolmeCommitReceiptFinality::Final,
        block_height: None,
        observed_at_unix_seconds: 1_900_000_070,
    }
}
