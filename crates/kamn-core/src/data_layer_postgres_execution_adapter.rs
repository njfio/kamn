//! Live PostgreSQL execution adapter for data-layer bridge descriptors.
//!
//! This module executes deterministic SQL descriptors emitted by
//! `data_layer_postgres_repository_bridge` and applies migration artifacts from
//! `crates/kamn-core/migrations`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::debug;

use crate::{
    data_layer_pg_project_blind_index_search_operation,
    data_layer_pg_project_default_rls_statements, data_layer_pg_project_insert_message_operation,
    data_layer_pg_project_select_message_by_id_operation, DataLayerM0EnvelopeRecord,
    DataLayerPgBlindIndexSearchRequest, DataLayerPgOperationKind, DataLayerPgRequesterSession,
};

mod codec;
mod error;
mod migration;
mod validation;

use codec::{
    data_layer_pg_decode_blind_index_search_row, data_layer_pg_decode_stored_message,
    data_layer_pg_encode_blind_indexes_json,
};
use migration::data_layer_pg_split_migration_statements;
use validation::{
    data_layer_pg_validate_non_empty_text, data_layer_pg_validate_positive_unix_timestamp,
    data_layer_pg_validate_uuid_text,
};

pub use error::DataLayerPgExecutionAdapterError;
pub use migration::data_layer_pg_collect_migration_files;

/// Stable reason marker for invalid adapter database URL inputs.
pub const DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE: &str =
    "data_layer_pg_execution_invalid_database_url";
/// Stable reason marker for migration-application failures.
pub const DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_migration_failed";
/// Stable reason marker for SQL execution failures.
pub const DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_sql_failed";
/// Stable reason marker for requester session setup failures.
pub const DATA_LAYER_PG_EXECUTION_SESSION_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_session_failed";
/// Stable reason marker for default RLS statement application failures.
const DATA_LAYER_PG_EXECUTION_RLS_FAILED_REASON_CODE: &str = "data_layer_pg_execution_rls_failed";
/// Stable reason marker for invalid merkle-batch payload inputs.
const DATA_LAYER_PG_EXECUTION_MERKLE_BATCH_PAYLOAD_FAILED_REASON_CODE: &str =
    "data_layer_pg_execution_invalid_merkle_batch_payload";

const DATA_LAYER_PG_MIGRATIONS_DIR: &str = "migrations";

/// Configuration inputs for constructing a live PostgreSQL execution adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgExecutionAdapterConfig {
    /// PostgreSQL connection URL.
    pub database_url: String,
    /// Pool max-connections setting.
    pub max_connections: u32,
}

impl DataLayerPgExecutionAdapterConfig {
    /// Creates a configuration with default pool-size controls.
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            max_connections: 4,
        }
    }
}

/// Deterministic migration execution report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgMigrationReport {
    /// Migration file names discovered in deterministic order.
    pub migration_files: Vec<String>,
}

/// Stored message row projection returned by lookup operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgStoredMessage {
    /// Message identifier.
    pub message_id: String,
    /// Owner DID.
    pub owner_did: String,
    /// Sender DID.
    pub sender_did: String,
    /// Recipient DID.
    pub recipient_did: String,
    /// Ciphertext payload.
    pub envelope_ciphertext: String,
    /// Content hash.
    pub content_hash_sha256: String,
    /// Previous hash-chain pointer.
    pub hash_chain_prev: String,
    /// Retention class marker.
    pub retention_class: String,
}

/// Blind-index search row projection returned by search operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgBlindIndexSearchRow {
    /// Message identifier.
    pub message_id: String,
    /// Owner DID.
    pub owner_did: String,
    /// Sender DID.
    pub sender_did: String,
    /// Recipient DID.
    pub recipient_did: String,
    /// Content hash marker.
    pub content_hash_sha256: String,
}

/// Deterministic default-RLS statement application report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsApplyReport {
    /// Statement outcomes in deterministic projection order.
    pub statement_outcomes: Vec<DataLayerPgRlsStatementOutcome>,
}

/// One applied RLS statement outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerPgRlsStatementOutcome {
    /// Target table name.
    pub table_name: String,
    /// Policy name marker.
    pub policy_name: String,
    /// Rows affected marker from PostgreSQL execution metadata.
    pub rows_affected: u64,
}

/// Live execution adapter for running PostgreSQL bridge descriptors.
#[derive(Debug, Clone)]
pub struct DataLayerPgExecutionAdapter {
    pool: PgPool,
}

impl DataLayerPgExecutionAdapter {
    /// Connects to PostgreSQL and creates an execution adapter.
    pub async fn connect(
        config: DataLayerPgExecutionAdapterConfig,
    ) -> Result<Self, DataLayerPgExecutionAdapterError> {
        if config.database_url.trim().is_empty() {
            return Err(DataLayerPgExecutionAdapterError::EmptyField("database_url"));
        }
        if config.max_connections == 0 {
            return Err(DataLayerPgExecutionAdapterError::InvalidMaxConnections(
                config.max_connections,
            ));
        }

        let options =
            PgConnectOptions::from_str(config.database_url.as_str()).map_err(|error| {
                DataLayerPgExecutionAdapterError::InvalidDatabaseUrl {
                    field: "database_url",
                    reason_code: DATA_LAYER_PG_EXECUTION_INVALID_DATABASE_URL_REASON_CODE,
                    detail: error.to_string(),
                }
            })?;
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::SelectMessageById,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: format!("connection failed: {error}"),
                },
            )?;

        Ok(Self { pool })
    }

    /// Applies embedded migration files in deterministic order.
    pub async fn apply_migrations(
        &self,
    ) -> Result<DataLayerPgMigrationReport, DataLayerPgExecutionAdapterError> {
        let migration_files = data_layer_pg_collect_migration_files()?;
        debug!(
            migration_count = migration_files.len(),
            "applying data-layer postgres migrations"
        );
        for migration_file in &migration_files {
            let migration_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(DATA_LAYER_PG_MIGRATIONS_DIR)
                .join(migration_file);
            let migration_source =
                fs::read_to_string(migration_path.as_path()).map_err(|error| {
                    DataLayerPgExecutionAdapterError::MigrationIoFailed {
                        reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                        detail: format!("failed reading {}: {error}", migration_path.display()),
                    }
                })?;
            let statements = data_layer_pg_split_migration_statements(migration_source.as_str());
            let mut transaction = self.pool.begin().await.map_err(|error| {
                DataLayerPgExecutionAdapterError::MigrationFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!(
                        "migration {} begin transaction failed: {error}",
                        migration_path.display()
                    ),
                }
            })?;
            for statement in statements {
                sqlx::query(statement.as_str())
                    .execute(&mut *transaction)
                    .await
                    .map_err(|error| DataLayerPgExecutionAdapterError::MigrationFailed {
                        reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                        detail: format!(
                            "migration {} statement failed: {error}",
                            migration_path.display()
                        ),
                    })?;
            }
            transaction.commit().await.map_err(|error| {
                DataLayerPgExecutionAdapterError::MigrationFailed {
                    reason_code: DATA_LAYER_PG_EXECUTION_MIGRATION_FAILED_REASON_CODE,
                    detail: format!(
                        "migration {} commit failed: {error}",
                        migration_path.display()
                    ),
                }
            })?;
        }

        Ok(DataLayerPgMigrationReport { migration_files })
    }

    /// Executes one message insert descriptor projected by the repository bridge.
    pub async fn execute_insert_message(
        &self,
        record: &DataLayerM0EnvelopeRecord,
        owner_did: &str,
        requester_did: &str,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        let empty_blind_indexes = BTreeMap::new();
        self.execute_insert_message_with_blind_indexes(
            record,
            owner_did,
            requester_did,
            &empty_blind_indexes,
        )
        .await
    }

    /// Executes one message insert descriptor with a caller-supplied blind-index token map.
    pub async fn execute_insert_message_with_blind_indexes(
        &self,
        record: &DataLayerM0EnvelopeRecord,
        owner_did: &str,
        requester_did: &str,
        blind_indexes: &BTreeMap<String, String>,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        let descriptor =
            data_layer_pg_project_insert_message_operation(record, owner_did, requester_did)
                .map_err(
                    |error| DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                        operation: DataLayerPgOperationKind::InsertMessage,
                        detail: error.to_string(),
                    },
                )?;
        let recipient_did = record.recipient_dids.first().ok_or(
            DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                detail: "recipient_dids must include at least one entry".to_owned(),
            },
        )?;
        let envelope_nonce = i64::try_from(record.envelope_nonce).map_err(|error| {
            DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                detail: format!("envelope_nonce conversion failed: {error}"),
            }
        })?;
        let blind_indexes_json = data_layer_pg_encode_blind_indexes_json(blind_indexes)?;

        debug!(
            operation = "insert_message",
            message_id = %record.message_id,
            requester_did = %descriptor.session.requester_did,
            "executing postgres insert descriptor"
        );

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::InsertMessage)
            .await?;
        self.apply_requester_session(
            &mut tx,
            &descriptor.session,
            DataLayerPgOperationKind::InsertMessage,
        )
        .await?;
        let execution = sqlx::query(descriptor.sql.as_str())
            .bind(record.message_id.as_str())
            .bind(owner_did)
            .bind(record.sender_did.as_str())
            .bind(recipient_did.as_str())
            .bind(record.envelope_ciphertext.as_bytes())
            .bind(envelope_nonce)
            .bind(record.content_hash.as_str())
            .bind(record.hash_chain_prev.as_str())
            .bind(blind_indexes_json.as_str())
            .bind(record.message_type.as_str())
            .execute(&mut *tx)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::InsertMessage,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::InsertMessage,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Executes one message lookup descriptor projected by the repository bridge.
    pub async fn execute_select_message_by_id(
        &self,
        message_id: &str,
        requester_did: &str,
    ) -> Result<Option<DataLayerPgStoredMessage>, DataLayerPgExecutionAdapterError> {
        let descriptor =
            data_layer_pg_project_select_message_by_id_operation(message_id, requester_did)
                .map_err(
                    |error| DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                        operation: DataLayerPgOperationKind::SelectMessageById,
                        detail: error.to_string(),
                    },
                )?;

        debug!(
            operation = "select_message_by_id",
            message_id = message_id,
            requester_did = %descriptor.session.requester_did,
            "executing postgres lookup descriptor"
        );

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::SelectMessageById)
            .await?;
        self.apply_requester_session(
            &mut tx,
            &descriptor.session,
            DataLayerPgOperationKind::SelectMessageById,
        )
        .await?;
        let row = sqlx::query(descriptor.sql.as_str())
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::SelectMessageById,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::SelectMessageById,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;

        row.map(data_layer_pg_decode_stored_message).transpose()
    }

    /// Executes one blind-index search descriptor projected by the repository bridge.
    pub async fn execute_search_messages_by_blind_index(
        &self,
        request: DataLayerPgBlindIndexSearchRequest,
    ) -> Result<Vec<DataLayerPgBlindIndexSearchRow>, DataLayerPgExecutionAdapterError> {
        let owner_did = request.owner_did.clone();
        let index_key = request.index_key.clone();
        let index_value_hash = request.index_value_hash.clone();
        let limit = i64::from(request.limit);
        let descriptor =
            data_layer_pg_project_blind_index_search_operation(request).map_err(|error| {
                DataLayerPgExecutionAdapterError::BridgeProjectionFailed {
                    operation: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
                    detail: error.to_string(),
                }
            })?;

        debug!(
            operation = "search_messages_by_blind_index",
            owner_did = %owner_did,
            requester_did = %descriptor.session.requester_did,
            "executing postgres blind-index search descriptor"
        );

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::SearchMessagesByBlindIndex)
            .await?;
        self.apply_requester_session(
            &mut tx,
            &descriptor.session,
            DataLayerPgOperationKind::SearchMessagesByBlindIndex,
        )
        .await?;
        let rows = sqlx::query(descriptor.sql.as_str())
            .bind(owner_did.as_str())
            .bind(index_key.as_str())
            .bind(index_value_hash.as_str())
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
                    reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::SearchMessagesByBlindIndex,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;

        rows.into_iter()
            .map(data_layer_pg_decode_blind_index_search_row)
            .collect()
    }

    /// Persists one merkle-batch row in scheduled status.
    pub async fn execute_create_merkle_batch(
        &self,
        batch_id: &str,
        root_hash: &str,
        leaf_count: i32,
        scheduled_at_unix_seconds: i64,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        data_layer_pg_validate_uuid_text(batch_id, "batch_id")?;
        data_layer_pg_validate_non_empty_text(root_hash, "root_hash")?;
        if leaf_count <= 0 {
            return Err(
                DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
                    field: "leaf_count",
                    detail: "must be greater than zero".to_owned(),
                },
            );
        }
        data_layer_pg_validate_positive_unix_timestamp(
            scheduled_at_unix_seconds,
            "scheduled_at_unix_seconds",
        )?;

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::InsertMerkleBatch)
            .await?;
        let execution = sqlx::query(
            "INSERT INTO merkle_batches (batch_id, root_hash, leaf_count, status, scheduled_at) VALUES ($1::uuid, $2, $3, 'scheduled', to_timestamp($4));",
        )
        .bind(batch_id)
        .bind(root_hash)
        .bind(leaf_count)
        .bind(scheduled_at_unix_seconds)
        .execute(&mut *tx)
        .await
        .map_err(|error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
            operation: DataLayerPgOperationKind::InsertMerkleBatch,
            reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
            detail: error.to_string(),
        })?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::InsertMerkleBatch,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Assigns one message row to a merkle batch and leaf index.
    pub async fn execute_assign_message_to_merkle_batch(
        &self,
        message_id: &str,
        batch_id: &str,
        merkle_leaf_index: i32,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        data_layer_pg_validate_uuid_text(message_id, "message_id")?;
        data_layer_pg_validate_uuid_text(batch_id, "batch_id")?;
        if merkle_leaf_index < 0 {
            return Err(
                DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
                    field: "merkle_leaf_index",
                    detail: "must be greater than or equal to zero".to_owned(),
                },
            );
        }

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::AssignMessageMerkleBatch)
            .await?;
        let execution = sqlx::query(
            "UPDATE messages SET merkle_batch_id = $1::uuid, merkle_leaf_index = $2 WHERE message_id = $3::uuid;",
        )
        .bind(batch_id)
        .bind(merkle_leaf_index)
        .bind(message_id)
        .execute(&mut *tx)
        .await
        .map_err(|error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
            operation: DataLayerPgOperationKind::AssignMessageMerkleBatch,
            reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
            detail: error.to_string(),
        })?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::AssignMessageMerkleBatch,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Marks a merkle batch as submitted and persists provider transaction metadata.
    pub async fn execute_mark_merkle_batch_submitted(
        &self,
        batch_id: &str,
        kolme_tx_hash: &str,
        submitted_at_unix_seconds: i64,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        data_layer_pg_validate_uuid_text(batch_id, "batch_id")?;
        data_layer_pg_validate_non_empty_text(kolme_tx_hash, "kolme_tx_hash")?;
        data_layer_pg_validate_positive_unix_timestamp(
            submitted_at_unix_seconds,
            "submitted_at_unix_seconds",
        )?;

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::MarkMerkleBatchSubmitted)
            .await?;
        let execution = sqlx::query(
            "UPDATE merkle_batches SET status = 'submitted', kolme_tx_hash = $2, submitted_at = to_timestamp($3) WHERE batch_id = $1::uuid;",
        )
        .bind(batch_id)
        .bind(kolme_tx_hash)
        .bind(submitted_at_unix_seconds)
        .execute(&mut *tx)
        .await
        .map_err(|error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
            operation: DataLayerPgOperationKind::MarkMerkleBatchSubmitted,
            reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
            detail: error.to_string(),
        })?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::MarkMerkleBatchSubmitted,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Marks a merkle batch as confirmed and persists finality metadata.
    pub async fn execute_mark_merkle_batch_confirmed(
        &self,
        batch_id: &str,
        kolme_block_height: i64,
        confirmed_at_unix_seconds: i64,
    ) -> Result<u64, DataLayerPgExecutionAdapterError> {
        data_layer_pg_validate_uuid_text(batch_id, "batch_id")?;
        if kolme_block_height < 0 {
            return Err(
                DataLayerPgExecutionAdapterError::InvalidMerkleBatchPayload {
                    field: "kolme_block_height",
                    detail: "must be greater than or equal to zero".to_owned(),
                },
            );
        }
        data_layer_pg_validate_positive_unix_timestamp(
            confirmed_at_unix_seconds,
            "confirmed_at_unix_seconds",
        )?;

        let mut tx = self
            .begin_transaction(DataLayerPgOperationKind::MarkMerkleBatchConfirmed)
            .await?;
        let execution = sqlx::query(
            "UPDATE merkle_batches SET status = 'confirmed', kolme_block_height = $2, confirmed_at = to_timestamp($3) WHERE batch_id = $1::uuid;",
        )
        .bind(batch_id)
        .bind(kolme_block_height)
        .bind(confirmed_at_unix_seconds)
        .execute(&mut *tx)
        .await
        .map_err(|error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
            operation: DataLayerPgOperationKind::MarkMerkleBatchConfirmed,
            reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
            detail: error.to_string(),
        })?;
        tx.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation: DataLayerPgOperationKind::MarkMerkleBatchConfirmed,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(execution.rows_affected())
    }

    /// Applies default M2 RLS policy statements in deterministic projection order.
    pub async fn apply_default_rls_statements(
        &self,
    ) -> Result<DataLayerPgRlsApplyReport, DataLayerPgExecutionAdapterError> {
        let statements = data_layer_pg_project_default_rls_statements();
        debug!(
            statement_count = statements.len(),
            "applying default data-layer postgres RLS statements"
        );
        let mut transaction = self.pool.begin().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::RlsStatementApplyFailed {
                reason_code: DATA_LAYER_PG_EXECUTION_RLS_FAILED_REASON_CODE,
                detail: format!("begin transaction failed: {error}"),
            }
        })?;
        let mut statement_outcomes = Vec::with_capacity(statements.len());
        for statement in statements {
            let execution = sqlx::query(statement.sql.as_str())
                .execute(&mut *transaction)
                .await
                .map_err(
                    |error| DataLayerPgExecutionAdapterError::RlsStatementApplyFailed {
                        reason_code: DATA_LAYER_PG_EXECUTION_RLS_FAILED_REASON_CODE,
                        detail: format!(
                            "table={} policy={} failed: {error}",
                            statement.table_name, statement.policy_name
                        ),
                    },
                )?;
            statement_outcomes.push(DataLayerPgRlsStatementOutcome {
                table_name: statement.table_name,
                policy_name: statement.policy_name,
                rows_affected: execution.rows_affected(),
            });
        }
        transaction.commit().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::RlsStatementApplyFailed {
                reason_code: DATA_LAYER_PG_EXECUTION_RLS_FAILED_REASON_CODE,
                detail: format!("commit failed: {error}"),
            }
        })?;
        Ok(DataLayerPgRlsApplyReport { statement_outcomes })
    }

    async fn begin_transaction(
        &self,
        operation: DataLayerPgOperationKind,
    ) -> Result<Transaction<'_, Postgres>, DataLayerPgExecutionAdapterError> {
        self.pool.begin().await.map_err(|error| {
            DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                operation,
                reason_code: DATA_LAYER_PG_EXECUTION_SQL_FAILED_REASON_CODE,
                detail: format!("begin transaction failed: {error}"),
            }
        })
    }

    async fn apply_requester_session(
        &self,
        transaction: &mut Transaction<'_, Postgres>,
        session: &DataLayerPgRequesterSession,
        operation: DataLayerPgOperationKind,
    ) -> Result<(), DataLayerPgExecutionAdapterError> {
        sqlx::query("SELECT set_config($1, $2, TRUE);")
            .bind(session.setting_key)
            .bind(session.requester_did.as_str())
            .execute(&mut **transaction)
            .await
            .map_err(
                |error| DataLayerPgExecutionAdapterError::SqlExecutionFailed {
                    operation,
                    reason_code: DATA_LAYER_PG_EXECUTION_SESSION_FAILED_REASON_CODE,
                    detail: error.to_string(),
                },
            )?;
        Ok(())
    }
}
