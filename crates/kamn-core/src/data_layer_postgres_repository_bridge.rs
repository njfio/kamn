//! PostgreSQL repository bridge contracts for data-layer persistence wiring.
//!
//! This module does not execute SQL. It projects validated data-layer inputs
//! into deterministic SQL operation descriptors that runtime adapters can
//! execute later.

use std::fmt;

use crate::{
    data_layer_m2_default_rls_policies, AgentDid, DataLayerM0EnvelopeRecord, KamnDid,
    DATA_LAYER_M2_REQUESTER_DID_SETTING,
};

/// Stable reason marker for invalid requester DID session inputs.
pub const DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE: &str =
    "data_layer_pg_invalid_requester_did";
/// Stable reason marker for invalid owner DID inputs.
pub const DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE: &str = "data_layer_pg_invalid_owner_did";

const DATA_LAYER_PG_MAX_BLIND_INDEX_SEARCH_LIMIT: u32 = 200;

/// Deterministic operation kind projected by the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerPgOperationKind {
    /// Insert one message row descriptor.
    InsertMessage,
    /// Select one message by id descriptor.
    SelectMessageById,
    /// Search message rows via blind-index descriptor.
    SearchMessagesByBlindIndex,
}

/// Requester session metadata projected into SQL execution context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRequesterSession {
    /// Session setting key used by RLS policy templates.
    pub setting_key: &'static str,
    /// Validated requester DID value.
    pub requester_did: String,
}

/// Deterministic SQL operation descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgSqlOperation {
    /// Operation kind.
    pub kind: DataLayerPgOperationKind,
    /// SQL statement text.
    pub sql: String,
    /// Stable bind-order markers.
    pub bind_markers: Vec<&'static str>,
    /// RLS requester session metadata.
    pub session: DataLayerPgRequesterSession,
}

/// Blind-index search request projected into SQL operation descriptors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgBlindIndexSearchRequest {
    /// Requester DID for RLS session context.
    pub requester_did: String,
    /// Owner DID used for owner-scope filtering.
    pub owner_did: String,
    /// Blind-index key.
    pub index_key: String,
    /// Blind-index token/hash value.
    pub index_value_hash: String,
    /// Maximum number of rows to return.
    pub limit: u32,
}

/// RLS SQL statement descriptor projected from M2 templates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsStatement {
    /// Target table name.
    pub table_name: String,
    /// Policy name tied to this statement.
    pub policy_name: String,
    /// SQL statement payload.
    pub sql: String,
}

/// Error taxonomy for PostgreSQL repository bridge projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPgRepositoryBridgeError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Requester DID failed validation.
    InvalidRequesterDid {
        /// Invalid input field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Owner DID failed validation.
    InvalidOwnerDid {
        /// Invalid input field name.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Parser detail.
        detail: String,
    },
    /// Search limit is outside accepted bounds.
    InvalidSearchLimit {
        /// Requested limit.
        requested: u32,
        /// Maximum accepted limit.
        max_allowed: u32,
    },
}

impl fmt::Display for DataLayerPgRepositoryBridgeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(formatter, "{field} must not be empty"),
            Self::InvalidRequesterDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid requester did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidOwnerDid {
                field,
                reason_code,
                detail,
            } => write!(
                formatter,
                "invalid owner did field {field}: {reason_code} ({detail})"
            ),
            Self::InvalidSearchLimit {
                requested,
                max_allowed,
            } => write!(
                formatter,
                "invalid blind-index search limit: requested {requested}, max {max_allowed}"
            ),
        }
    }
}

impl std::error::Error for DataLayerPgRepositoryBridgeError {}

/// Projects a deterministic insert-message SQL operation descriptor.
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

    let sql = "INSERT INTO messages (message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, envelope_nonce, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10);";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertMessage,
        sql: sql.to_owned(),
        bind_markers: vec![
            "message_id",
            "owner_did",
            "sender_did",
            "recipient_did",
            "envelope_ciphertext",
            "envelope_nonce",
            "content_hash_sha256",
            "hash_chain_prev",
            "blind_indexes",
            "retention_class",
        ],
        session,
    })
}

/// Projects a deterministic message lookup SQL operation descriptor.
pub fn data_layer_pg_project_select_message_by_id_operation(
    message_id: &str,
    requester_did: &str,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_non_empty(message_id, "message_id")?;
    let session = build_requester_session(requester_did)?;

    let sql = "SELECT message_id, owner_did, sender_did, recipient_did, envelope_ciphertext, content_hash_sha256, hash_chain_prev, blind_indexes, retention_class, shredded_at, created_at FROM messages WHERE message_id = $1;";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SelectMessageById,
        sql: sql.to_owned(),
        bind_markers: vec!["message_id"],
        session,
    })
}

/// Projects a deterministic blind-index search SQL operation descriptor.
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

    let sql = "SELECT message_id, owner_did, sender_did, recipient_did, content_hash_sha256, created_at FROM messages WHERE owner_did = $1 AND blind_indexes ->> $2 = $3 ORDER BY created_at DESC LIMIT $4;";

    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
        sql: sql.to_owned(),
        bind_markers: vec!["owner_did", "index_key", "index_value_hash", "limit"],
        session,
    })
}

/// Projects default M2 RLS templates into deterministic SQL statement descriptors.
pub fn data_layer_pg_project_default_rls_statements() -> Vec<DataLayerPgRlsStatement> {
    let mut policies = data_layer_m2_default_rls_policies();
    policies.sort_by(|left, right| {
        left.table_name
            .cmp(&right.table_name)
            .then(left.policy_name.cmp(&right.policy_name))
    });

    let mut statements = Vec::with_capacity(policies.len() * 3);
    for policy in policies {
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;",
                policy.table_name
            ),
        });
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name.clone(),
            policy_name: policy.policy_name.clone(),
            sql: format!(
                "DROP POLICY IF EXISTS {} ON {};",
                policy.policy_name, policy.table_name
            ),
        });
        let mut create_sql = format!(
            "CREATE POLICY {} ON {} USING ({}",
            policy.policy_name, policy.table_name, policy.using_clause
        );
        create_sql.push(')');
        if let Some(with_check_clause) = policy.with_check_clause {
            create_sql.push_str(format!(" WITH CHECK ({with_check_clause})").as_str());
        }
        create_sql.push(';');
        statements.push(DataLayerPgRlsStatement {
            table_name: policy.table_name,
            policy_name: policy.policy_name,
            sql: create_sql,
        });
    }

    statements
}

fn build_requester_session(
    requester_did: &str,
) -> Result<DataLayerPgRequesterSession, DataLayerPgRepositoryBridgeError> {
    let parsed = AgentDid::parse(requester_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidRequesterDid {
            field: "requester_did",
            reason_code: DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(DataLayerPgRequesterSession {
        setting_key: DATA_LAYER_M2_REQUESTER_DID_SETTING,
        requester_did: parsed.as_str().to_owned(),
    })
}

fn validate_owner_did(owner_did: &str) -> Result<(), DataLayerPgRepositoryBridgeError> {
    KamnDid::parse(owner_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidOwnerDid {
            field: "owner_did",
            reason_code: DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(())
}

fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if value.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(field));
    }
    Ok(())
}
