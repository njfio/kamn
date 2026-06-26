use crate::support::{
    blind_index_map, blind_index_request, blind_index_token, connect_live_adapter, current_suffix,
    fixture_record, live_postgres_url, runtime, uuid_from_u128,
};

#[test]
fn spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        let message_id = uuid_from_u128(current_suffix());
        let record = fixture_record(message_id.clone());
        insert_fixture_message(&adapter, &record).await;
        assert_stored_message(&adapter, &message_id).await;
    });
}

#[test]
fn spec_c01_live_adapter_executes_blind_index_search_with_session_context() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        let results = adapter
            .execute_search_messages_by_blind_index(blind_index_request(
                "missing-index-token".to_owned(),
            ))
            .await
            .expect("search execution should succeed");
        assert!(results.is_empty());
    });
}

#[test]
fn spec_c03_live_adapter_persists_blind_indexes_on_insert_and_search_retrieves_row() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = connect_live_adapter(database_url).await;
        let message_id = uuid_from_u128(current_suffix());
        let record = fixture_record(message_id.clone());
        let token = blind_index_token();
        insert_message_with_index(&adapter, &record, token.clone()).await;
        assert_index_search_includes_message(&adapter, token, &message_id).await;
    });
}

async fn insert_fixture_message(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    record: &kamn_core::DataLayerM0EnvelopeRecord,
) {
    let inserted = adapter
        .execute_insert_message(record, "kamn:did:owner:owner-1", "kamn:did:agent:agent-1")
        .await
        .expect("insert should succeed");
    assert_eq!(inserted, 1);
}

async fn assert_stored_message(adapter: &kamn_core::DataLayerPgExecutionAdapter, message_id: &str) {
    let stored = adapter
        .execute_select_message_by_id(message_id, "kamn:did:agent:agent-1")
        .await
        .expect("lookup should succeed")
        .expect("inserted row should resolve");
    assert_eq!(stored.message_id, message_id);
    assert_eq!(stored.owner_did, "kamn:did:owner:owner-1");
}

async fn insert_message_with_index(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    record: &kamn_core::DataLayerM0EnvelopeRecord,
    token: String,
) {
    let inserted = adapter
        .execute_insert_message_with_blind_indexes(
            record,
            "kamn:did:owner:owner-1",
            "kamn:did:agent:agent-1",
            &blind_index_map(token),
        )
        .await
        .expect("insert with blind-index map should succeed");
    assert_eq!(inserted, 1);
}

async fn assert_index_search_includes_message(
    adapter: &kamn_core::DataLayerPgExecutionAdapter,
    token: String,
    message_id: &str,
) {
    let search_results = adapter
        .execute_search_messages_by_blind_index(blind_index_request(token))
        .await
        .expect("search should succeed");
    assert!(search_results
        .iter()
        .any(|row| row.message_id == message_id));
}
