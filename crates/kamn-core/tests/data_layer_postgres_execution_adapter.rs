use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_core::{
    data_layer_m3_compute_blind_index, data_layer_pg_collect_migration_files,
    DataLayerM0EnvelopeRecord, DataLayerM0WrappedKey, DataLayerPgBlindIndexSearchRequest,
    DataLayerPgExecutionAdapter, DataLayerPgExecutionAdapterConfig,
    DataLayerPgExecutionAdapterError, DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE,
};

fn live_postgres_url() -> Option<String> {
    std::env::var("KAMN_TEST_POSTGRES_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .filter(|value| !value.trim().is_empty())
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible")
}

fn fixture_record(message_id: String) -> DataLayerM0EnvelopeRecord {
    DataLayerM0EnvelopeRecord {
        message_id,
        content_hash: "sha256:content-hash".to_owned(),
        hash_chain_prev: "sha256:prev".to_owned(),
        sender_did: "kamn:did:agent:sender-1".to_owned(),
        recipient_dids: vec!["kamn:did:agent:recipient-1".to_owned()],
        message_type: "standard".to_owned(),
        envelope_ciphertext: "ciphertext-blob".to_owned(),
        envelope_nonce: 42,
        envelope_aad_hash: "sha256:aad".to_owned(),
        wrapped_keys: vec![DataLayerM0WrappedKey {
            did: "kamn:did:agent:recipient-1".to_owned(),
            wrapped_cek: "wrapped-cek".to_owned(),
        }],
        compression_codec: "zstd".to_owned(),
        compression_dict_id: Some(7),
        content_size_bytes: 2048,
        compressed_size_bytes: 512,
    }
}

#[test]
fn spec_c02_migration_file_inventory_is_deterministic() {
    let files =
        data_layer_pg_collect_migration_files().expect("migration inventory should be readable");
    assert_eq!(
        files,
        vec!["202602190001_data_layer_phase1_bootstrap.sql".to_owned()],
        "migration inventory should be deterministic and ordered"
    );
}

#[test]
fn spec_c04_invalid_database_url_fails_closed() {
    let runtime = runtime();
    let error = runtime
        .block_on(DataLayerPgExecutionAdapter::connect(
            DataLayerPgExecutionAdapterConfig::new("not-a-valid-url"),
        ))
        .expect_err("invalid URL must fail closed");
    match error {
        DataLayerPgExecutionAdapterError::InvalidDatabaseUrl {
            field, reason_code, ..
        } => {
            assert_eq!(field, "database_url");
            assert_eq!(
                reason_code,
                DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE
            );
        }
        other => panic!("unexpected error variant: {other:?}"),
    }
}

#[test]
fn spec_c01_and_c03_live_adapter_executes_insert_and_lookup_with_session_context() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = DataLayerPgExecutionAdapter::connect(DataLayerPgExecutionAdapterConfig {
            database_url,
            max_connections: 4,
        })
        .await
        .expect("live postgres connection should succeed when test URL is provided");

        adapter
            .apply_migrations()
            .await
            .expect("migrations should apply before execution");

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let message_id = format!("msg-live-{suffix}");
        let record = fixture_record(message_id.clone());

        let inserted = adapter
            .execute_insert_message(&record, "kamn:did:owner:owner-1", "kamn:did:agent:agent-1")
            .await
            .expect("insert should succeed");
        assert_eq!(inserted, 1, "insert should affect exactly one row");

        let stored = adapter
            .execute_select_message_by_id(message_id.as_str(), "kamn:did:agent:agent-1")
            .await
            .expect("lookup should succeed")
            .expect("inserted row should resolve");

        assert_eq!(stored.message_id, message_id);
        assert_eq!(stored.owner_did, "kamn:did:owner:owner-1");
    });
}

#[test]
fn spec_c01_live_adapter_executes_blind_index_search_with_session_context() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = DataLayerPgExecutionAdapter::connect(DataLayerPgExecutionAdapterConfig {
            database_url,
            max_connections: 4,
        })
        .await
        .expect("live postgres connection should succeed when test URL is provided");

        adapter
            .apply_migrations()
            .await
            .expect("migrations should apply before execution");

        let results = adapter
            .execute_search_messages_by_blind_index(DataLayerPgBlindIndexSearchRequest {
                requester_did: "kamn:did:agent:agent-1".to_owned(),
                owner_did: "kamn:did:owner:owner-1".to_owned(),
                index_key: "channel_topic".to_owned(),
                index_value_hash: "missing-index-token".to_owned(),
                limit: 10,
            })
            .await
            .expect("search execution should succeed");
        assert!(
            results.is_empty(),
            "search over empty/missing blind index should return deterministic empty result set"
        );
    });
}

#[test]
fn spec_c02_live_adapter_applies_default_rls_statements_deterministically() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = DataLayerPgExecutionAdapter::connect(DataLayerPgExecutionAdapterConfig {
            database_url,
            max_connections: 4,
        })
        .await
        .expect("live postgres connection should succeed when test URL is provided");

        adapter
            .apply_migrations()
            .await
            .expect("migrations should apply before execution");

        let first_report = adapter
            .apply_default_rls_statements()
            .await
            .expect("default RLS policies should apply");
        let second_report = adapter
            .apply_default_rls_statements()
            .await
            .expect("default RLS policies should be idempotent");

        assert!(
            !first_report.statement_outcomes.is_empty(),
            "RLS apply report should include deterministic statement outcomes"
        );
        assert_eq!(
            first_report.statement_outcomes.len(),
            second_report.statement_outcomes.len(),
            "idempotent reapplies should execute the same number of statements"
        );
    });
}

#[test]
fn spec_c03_live_adapter_persists_blind_indexes_on_insert_and_search_retrieves_row() {
    let Some(database_url) = live_postgres_url() else {
        return;
    };

    let runtime = runtime();
    runtime.block_on(async move {
        let adapter = DataLayerPgExecutionAdapter::connect(DataLayerPgExecutionAdapterConfig {
            database_url,
            max_connections: 4,
        })
        .await
        .expect("live postgres connection should succeed when test URL is provided");

        adapter
            .apply_migrations()
            .await
            .expect("migrations should apply before execution");

        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let message_id = format!("msg-live-blind-index-{suffix}");
        let record = fixture_record(message_id.clone());
        let blind_index_token =
            data_layer_m3_compute_blind_index("owner-phase2-key", "channel_topic", "alpha")
                .expect("blind-index token derivation should succeed");
        let mut blind_indexes = BTreeMap::new();
        blind_indexes.insert("channel_topic".to_owned(), blind_index_token.clone());

        let inserted = adapter
            .execute_insert_message_with_blind_indexes(
                &record,
                "kamn:did:owner:owner-1",
                "kamn:did:agent:agent-1",
                &blind_indexes,
            )
            .await
            .expect("insert with blind-index map should succeed");
        assert_eq!(inserted, 1, "insert should affect exactly one row");

        let search_results = adapter
            .execute_search_messages_by_blind_index(DataLayerPgBlindIndexSearchRequest {
                requester_did: "kamn:did:agent:agent-1".to_owned(),
                owner_did: "kamn:did:owner:owner-1".to_owned(),
                index_key: "channel_topic".to_owned(),
                index_value_hash: blind_index_token,
                limit: 10,
            })
            .await
            .expect("search should succeed");
        assert!(
            search_results
                .iter()
                .any(|row| row.message_id == message_id),
            "search result should include inserted message keyed by blind-index token"
        );
    });
}
