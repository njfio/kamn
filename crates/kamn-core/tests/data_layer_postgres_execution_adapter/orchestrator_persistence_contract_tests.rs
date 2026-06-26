use kamn_core::{
    reconcile_data_layer_m1_finality_observation, DataLayerM1AnchoringFinalityObservation,
    DataLayerM1AnchoringFinalityReconciliationProjection, DataLayerM1AnchoringFollowUpAction,
    DataLayerM1AnchoringOrchestrator, DataLayerM1AnchoringPersistencePlan,
    DataLayerM1AnchoringTickOutcome, DataLayerM1BatchSchedulerPolicy,
    InMemoryKolmeRuntimeCommitClient, KolmeCommitReceiptFinality,
    DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE,
};

use crate::support::{
    connect_live_adapter, current_suffix, fixture_record, insert_fixture_message,
    live_postgres_url, pending_message, runtime, uuid_from_u128,
};

#[test]
fn spec_c03_live_orchestrator_plan_applies_via_adapter_lifecycle_methods() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        let message_id = uuid_from_u128(current_suffix() + 200);
        let record = fixture_record(message_id.clone());

        insert_fixture_message(&adapter, &record).await;
        let mut orchestrator = build_orchestrator();
        let outcome = plan_outcome(&mut orchestrator, &message_id, &record.content_hash);
        let final_projection = reconcile_projection(&outcome);
        let persistence_plan = planned_persistence_plan(&outcome);
        assert_follow_up_policy(&outcome);
        persist_orchestrator_plan(&adapter, persistence_plan).await;
        persist_confirmation(
            &adapter,
            persistence_plan.batch_id.as_str(),
            &final_projection,
        )
        .await;
    });
}

fn build_orchestrator() -> DataLayerM1AnchoringOrchestrator<InMemoryKolmeRuntimeCommitClient> {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy =
        DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be constructible");
    DataLayerM1AnchoringOrchestrator::new(
        client,
        "kamn:did:agent:orchestrator-live-c03",
        "merkle-anchor-root",
        policy,
    )
    .expect("orchestrator should initialize")
}

fn plan_outcome(
    orchestrator: &mut DataLayerM1AnchoringOrchestrator<InMemoryKolmeRuntimeCommitClient>,
    message_id: &str,
    content_hash: &str,
) -> DataLayerM1AnchoringTickOutcome {
    orchestrator
        .plan_tick(
            &[pending_message(message_id, content_hash, 1_900_000_000 - 5)],
            1_900_000_000,
            1_900_000_000,
            1_900_000_010,
            None,
        )
        .expect("orchestrator tick should evaluate")
}

fn reconcile_projection(
    outcome: &DataLayerM1AnchoringTickOutcome,
) -> DataLayerM1AnchoringFinalityReconciliationProjection {
    reconcile_data_layer_m1_finality_observation(
        outcome,
        &DataLayerM1AnchoringFinalityObservation {
            provider: "kolme-memory".to_owned(),
            transaction_id: "commit-1".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
            block_height: Some(123_456),
            observed_at_unix_seconds: 1_900_000_090,
        },
    )
    .expect("finality reconciliation should succeed")
}

fn planned_persistence_plan(
    outcome: &DataLayerM1AnchoringTickOutcome,
) -> &DataLayerM1AnchoringPersistencePlan {
    let DataLayerM1AnchoringTickOutcome::Planned {
        persistence_plan, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    persistence_plan
}

fn assert_follow_up_policy(outcome: &DataLayerM1AnchoringTickOutcome) {
    let DataLayerM1AnchoringTickOutcome::Planned {
        follow_up_policy, ..
    } = outcome
    else {
        panic!("expected planned outcome");
    };
    assert_eq!(
        follow_up_policy.action,
        DataLayerM1AnchoringFollowUpAction::PollConfirmation
    );
    assert_eq!(
        follow_up_policy.reason_code,
        DATA_LAYER_M1_ANCHORING_FOLLOW_UP_POLL_PENDING_REASON_CODE
    );
}

async fn persist_orchestrator_plan(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    persistence_plan: &DataLayerM1AnchoringPersistencePlan,
) {
    persist_batch_create(adapter, persistence_plan).await;
    persist_assignments(adapter, persistence_plan).await;
    persist_submission(adapter, persistence_plan).await;
}

async fn persist_batch_create(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    persistence_plan: &DataLayerM1AnchoringPersistencePlan,
) {
    let created = adapter
        .execute_create_merkle_batch(
            &persistence_plan.batch_id,
            &persistence_plan.merkle_root,
            persistence_plan.leaf_count,
            persistence_plan.scheduled_at_unix_seconds,
        )
        .await
        .expect("batch creation should succeed");
    assert_eq!(created, 1);
}

async fn persist_assignments(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    persistence_plan: &DataLayerM1AnchoringPersistencePlan,
) {
    let mut assigned_total = 0u64;
    for assignment in &persistence_plan.assignments {
        assigned_total += adapter
            .execute_assign_message_to_merkle_batch(
                &assignment.message_id,
                &persistence_plan.batch_id,
                assignment.leaf_index,
            )
            .await
            .expect("assignment should succeed");
    }
    assert_eq!(assigned_total, persistence_plan.assignments.len() as u64);
}

async fn persist_submission(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    persistence_plan: &DataLayerM1AnchoringPersistencePlan,
) {
    let submission = persistence_plan
        .submission
        .as_ref()
        .expect("planned outcome should include submission metadata");
    let submitted = adapter
        .execute_mark_merkle_batch_submitted(
            &persistence_plan.batch_id,
            &submission.kolme_tx_hash,
            submission.submitted_at_unix_seconds,
        )
        .await
        .expect("submission persistence should succeed");
    assert_eq!(submitted, 1);
}

async fn persist_confirmation(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    batch_id: &str,
    final_projection: &DataLayerM1AnchoringFinalityReconciliationProjection,
) {
    let confirmation = final_projection
        .confirmation
        .as_ref()
        .expect("final reconciliation should project confirmation metadata");
    let confirmed = adapter
        .execute_mark_merkle_batch_confirmed(
            batch_id,
            confirmation.kolme_block_height,
            confirmation.confirmed_at_unix_seconds,
        )
        .await
        .expect("confirmation persistence should succeed");
    assert_eq!(confirmed, 1);
}
