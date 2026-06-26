use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_core::{
    data_layer_m3_compute_blind_index, DataLayerM0EnvelopeRecord, DataLayerM0WrappedKey,
    DataLayerM1PendingBatchMessage, DataLayerPgBlindIndexSearchRequest,
    DataLayerPgExecutionAdapter, DataLayerPgExecutionAdapterConfig,
};

pub(crate) fn live_postgres_url() -> Option<String> {
    std::env::var("KAMN_TEST_POSTGRES_URL")
        .ok()
        .or_else(|| std::env::var("DATABASE_URL").ok())
        .filter(|value| !value.trim().is_empty())
}

pub(crate) fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime should be constructible")
}

pub(crate) fn uuid_from_u128(seed: u128) -> String {
    let hex = format!("{seed:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

pub(crate) fn current_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

pub(crate) fn pending_message(
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

pub(crate) fn fixture_record(message_id: String) -> DataLayerM0EnvelopeRecord {
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

pub(crate) async fn connect_live_adapter(database_url: String) -> DataLayerPgExecutionAdapter {
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
    adapter
}

pub(crate) async fn insert_fixture_message(
    adapter: &DataLayerPgExecutionAdapter,
    record: &DataLayerM0EnvelopeRecord,
) {
    adapter
        .execute_insert_message(record, "kamn:did:owner:owner-1", "kamn:did:agent:agent-1")
        .await
        .expect("insert should succeed");
}

pub(crate) fn blind_index_map(token: String) -> BTreeMap<String, String> {
    let mut blind_indexes = BTreeMap::new();
    blind_indexes.insert("channel_topic".to_owned(), token);
    blind_indexes
}

pub(crate) fn blind_index_token() -> String {
    data_layer_m3_compute_blind_index("owner-phase2-key", "channel_topic", "alpha")
        .expect("blind-index token derivation should succeed")
}

pub(crate) fn blind_index_request(token: String) -> DataLayerPgBlindIndexSearchRequest {
    DataLayerPgBlindIndexSearchRequest {
        requester_did: "kamn:did:agent:agent-1".to_owned(),
        owner_did: "kamn:did:owner:owner-1".to_owned(),
        index_key: "channel_topic".to_owned(),
        index_value_hash: token,
        limit: 10,
    }
}
