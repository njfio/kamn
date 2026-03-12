use crate::DataLayerM0EnvelopeRecord;

use super::super::{
    build_requester_session, validate_non_empty, validate_owner_did,
    DataLayerPgBlindIndexSearchRequest, DataLayerPgOperationKind, DataLayerPgRepositoryBridgeError,
    DataLayerPgSqlOperation, DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT,
};

pub fn data_layer_pg_project_insert_message_operation(
    record: &DataLayerM0EnvelopeRecord,
    owner_did: &str,
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(record.message_id.as_str(), "message_id")?;
    validate_non_empty(record.content_hash.as_str(), "content_hash")?;
    validate_non_empty(record.hash_chain_prev.as_str(), "hash_chain_prev")?;
    validate_non_empty(record.envelope_ciphertext.as_str(), "envelope_ciphertext")?;
    if record.recipient_dids.is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "recipient_dids",
        ));
    }
    validate_owner_did(owner_did)?;
    let session = build_requester_session(requester_did)?;
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertMessage,
        sql: "INSERT INTO messages (message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, envelope_nonce, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10);".to_owned(),
        bind_markers: vec!["message_id", "owner_did", "sender_did", "recipient_did", "envelope_ciphertext", "envelope_nonce", "content_hash_sha256", "hash_chain_prev", "blind_indexes", "retention_class"],
        session,
    })
}

pub fn data_layer_pg_project_select_message_by_id_operation(
    message_id: &str,
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(message_id, "message_id")?;
    let session = build_requester_session(requester_did)?;
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SelectMessageById,
        sql: "SELECT message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class, shredded_at, created_at FROM messages WHERE message_id = $1;".to_owned(),
        bind_markers: vec!["message_id"],
        session,
    })
}

pub fn data_layer_pg_project_blind_index_search_operation(
    request: DataLayerPgBlindIndexSearchRequest,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(request.owner_did.as_str(), "owner_did")?;
    validate_non_empty(request.index_key.as_str(), "index_key")?;
    validate_non_empty(request.index_value_hash.as_str(), "index_value_hash")?;
    validate_owner_did(request.owner_did.as_str())?;
    if request.limit == 0 || request.limit > DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: request.limit,
            max_allowed: DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT,
        });
    }
    let session = build_requester_session(request.requester_did.as_str())?;
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
        sql: "SELECT message_id, owner_did, sender_did, recipient_did, content_hash_sha256, created_at FROM messages WHERE owner_did = $1 AND blind_indexes ->> $2 = $3 ORDER BY created_at DESC LIMIT $4;".to_owned(),
        bind_markers: vec!["owner_did", "index_key", "index_value_hash", "limit"],
        session,
    })
}
