use kamn_core::DataLayerPgExecutionAdapterError;

use crate::support::{
    connect_live_adapter, current_suffix, fixture_record, live_postgres_url, runtime,
    uuid_from_u128,
};

#[test]
fn spec_c03_live_adapter_persists_merkle_batch_assignment_and_lifecycle_transitions() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        let suffix = current_suffix();
        let message_id = uuid_from_u128(suffix);
        let batch_id = uuid_from_u128(suffix + 1);
        let record = fixture_record(message_id.clone());

        insert_message(&adapter, &record).await;
        assert_batch_create(&adapter, &batch_id).await;
        assert_assignment(&adapter, &message_id, &batch_id).await;
        assert_submitted(&adapter, &batch_id).await;
        assert_confirmed(&adapter, &batch_id).await;
    });
}

#[test]
fn spec_c04_merkle_batch_lifecycle_fails_closed_for_invalid_payloads() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        assert_invalid_batch_id(&adapter).await;
        assert_invalid_confirm_height(&adapter).await;
    });
}

async fn insert_message(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    record: &kamn_core::DataLayerM0EnvelopeRecord,
) {
    adapter
        .execute_insert_message(record, "kamn:did:owner:owner-1", "kamn:did:agent:agent-1")
        .await
        .expect("insert should succeed");
}

async fn assert_batch_create(adapter: &kamn_core::DataLayerPgExecutionAdapter, batch_id: &str) {
    let created = adapter
        .execute_create_merkle_batch(batch_id, "sha256:merkle-root-c03", 1, 1_900_000_000)
        .await
        .expect("merkle batch create should succeed");
    assert_eq!(created, 1);
}

async fn assert_assignment(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    message_id: &str,
    batch_id: &str,
) {
    let assigned = adapter
        .execute_assign_message_to_merkle_batch(message_id, batch_id, 0)
        .await
        .expect("message assignment should succeed");
    assert_eq!(assigned, 1);
}

async fn assert_submitted(adapter: &kamn_core::DataLayerPgExecutionAdapter, batch_id: &str) {
    let submitted = adapter
        .execute_mark_merkle_batch_submitted(batch_id, "tx-c03", 1_900_000_010)
        .await
        .expect("submitted transition should succeed");
    assert_eq!(submitted, 1);
}

async fn assert_confirmed(adapter: &kamn_core::DataLayerPgExecutionAdapter, batch_id: &str) {
    let confirmed = adapter
        .execute_mark_merkle_batch_confirmed(batch_id, 123_456, 1_900_000_020)
        .await
        .expect("confirmed transition should succeed");
    assert_eq!(confirmed, 1);
}

async fn assert_invalid_batch_id(adapter: &kamn_core::DataLayerPgExecutionAdapter) {
    let invalid_batch = adapter
        .execute_create_merkle_batch("not-a-uuid", "sha256:root", 1, 1_900_000_000)
        .await
        .expect_err("invalid batch id should fail closed");
    assert!(matches!(
        invalid_batch,
        DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
            field: "batch_id",
            ..
        }
    ));
}

async fn assert_invalid_confirm_height(adapter: &kamn_core::DataLayerPgExecutionAdapter) {
    let invalid_confirm = adapter
        .execute_mark_merkle_batch_confirmed(
            "00000000-0000-0000-0000-000000000000",
            -1,
            1_900_000_020,
        )
        .await
        .expect_err("negative block height should fail closed");
    assert!(matches!(
        invalid_confirm,
        DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
            field: "kolme_block_height",
            ..
        }
    ));
}
