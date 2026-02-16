use crate::config::NodeRole;
use crate::sqlite_store_backend::{SqliteStoreBackend, SqliteStoreBackendError};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use super::validation::validate_canonical_commit_store_field;
use super::{BlockPipelineError, CanonicalCommitRecord};

/// Canonical commit persistence interface used by transport-fed block pipeline.
pub trait CanonicalCommitStore {
    /// Persists canonical commit record after fork-choice acceptance.
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError>;

    /// Lists canonical commit records in persistence order.
    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// In-memory canonical commit store for deterministic tests and local runtime probes.
pub struct InMemoryCanonicalCommitStore {
    records: Vec<CanonicalCommitRecord>,
}

impl CanonicalCommitStore for InMemoryCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        self.records.push(record);
        Ok(())
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        Ok(self.records.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// File-backed canonical commit store for restart/replay persistence checks.
pub struct FileCanonicalCommitStore {
    path: PathBuf,
}

impl FileCanonicalCommitStore {
    /// Creates file-backed canonical commit store from path.
    pub fn new(path: PathBuf) -> Result<Self, BlockPipelineError> {
        if path.as_os_str().is_empty() {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit store path is empty (canonical_commit_store_path_invalid)"
                    .to_owned(),
            ));
        }
        Ok(Self { path })
    }
}

impl CanonicalCommitStore for FileCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        let existing = self.list_canonical_commits()?;
        if let Some(last) = existing.last() {
            if record.block_height <= last.block_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit block height regression: previous {}, found {} (canonical_commit_store_block_height_regression)",
                    last.block_height, record.block_height
                )));
            }
        }

        let serialized = serialize_canonical_commit_record(&record)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit store append failed: {error} (canonical_commit_store_io)"
                ))
            })?;
        file.write_all(serialized.as_bytes()).map_err(|error| {
            BlockPipelineError::CommitStore(format!(
                "canonical commit store write failed: {error} (canonical_commit_store_io)"
            ))
        })
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let payload = fs::read_to_string(&self.path).map_err(|error| {
            BlockPipelineError::CommitStore(format!(
                "canonical commit store read failed: {error} (canonical_commit_store_io)"
            ))
        })?;
        let mut records: Vec<CanonicalCommitRecord> = Vec::new();
        for raw_line in payload.lines() {
            let line = raw_line.trim();
            if line.is_empty() {
                continue;
            }
            let record = parse_canonical_commit_record(line)?;
            if let Some(previous) = records.last() {
                if record.block_height <= previous.block_height {
                    return Err(BlockPipelineError::CommitStore(format!(
                        "canonical commit block height regression in persisted lineage: previous {}, found {} (canonical_commit_store_block_height_regression)",
                        previous.block_height, record.block_height
                    )));
                }
            }
            records.push(record);
        }
        Ok(records)
    }
}

const CANONICAL_COMMIT_SQLITE_META_NAMESPACE: &str = "canonical_commit_store_meta";
const CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE: &str = "canonical_commit_store_entries";
const CANONICAL_COMMIT_SQLITE_SCHEMA_KEY: &str = "schema_version";
const CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug)]
/// Sqlite-backed canonical commit store with strict schema/version guards.
pub struct SqliteCanonicalCommitStore {
    backend: SqliteStoreBackend,
}

impl SqliteCanonicalCommitStore {
    /// Creates sqlite-backed canonical commit store from path.
    pub fn new(path: PathBuf) -> Result<Self, BlockPipelineError> {
        let backend =
            SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_commit_store_error)?;
        let mut store = Self { backend };
        store.bootstrap_and_validate_schema_version()?;
        Ok(store)
    }

    fn bootstrap_and_validate_schema_version(&mut self) -> Result<(), BlockPipelineError> {
        let current = self
            .backend
            .get(
                CANONICAL_COMMIT_SQLITE_META_NAMESPACE,
                CANONICAL_COMMIT_SQLITE_SCHEMA_KEY,
            )
            .map_err(map_sqlite_commit_store_error)?;

        if let Some(bytes) = current {
            let schema_raw = String::from_utf8(bytes).map_err(|_| {
                BlockPipelineError::CommitStore(
                    "canonical commit sqlite schema value is not utf-8 (canonical_commit_store_sqlite_schema_invalid)"
                        .to_owned(),
                )
            })?;
            let found = schema_raw.parse::<u32>().map_err(|_| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite schema value is invalid: {schema_raw} (canonical_commit_store_sqlite_schema_invalid)"
                ))
            })?;
            if found != CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite schema mismatch: expected {}, found {} (canonical_commit_store_sqlite_schema_mismatch)",
                    CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION, found
                )));
            }
            return Ok(());
        }

        let existing_keys = self
            .backend
            .list_keys(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE)
            .map_err(map_sqlite_commit_store_error)?;
        if !existing_keys.is_empty() {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit sqlite schema row missing with existing commit entries (canonical_commit_store_sqlite_schema_missing)"
                    .to_owned(),
            ));
        }

        self.backend
            .put(
                CANONICAL_COMMIT_SQLITE_META_NAMESPACE,
                CANONICAL_COMMIT_SQLITE_SCHEMA_KEY,
                CANONICAL_COMMIT_SQLITE_SCHEMA_VERSION
                    .to_string()
                    .as_bytes(),
            )
            .map_err(map_sqlite_commit_store_error)
    }
}

impl CanonicalCommitStore for SqliteCanonicalCommitStore {
    fn persist_canonical_commit(
        &mut self,
        record: CanonicalCommitRecord,
    ) -> Result<(), BlockPipelineError> {
        let existing = self.list_canonical_commits()?;
        if let Some(last) = existing.last() {
            if record.block_height <= last.block_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit block height regression: previous {}, found {} (canonical_commit_store_block_height_regression)",
                    last.block_height, record.block_height
                )));
            }
        }

        let key = sqlite_canonical_commit_store_key(record.block_height);
        let payload = serialize_canonical_commit_record(&record)?;
        self.backend
            .put(
                CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE,
                key.as_str(),
                payload.as_bytes(),
            )
            .map_err(map_sqlite_commit_store_error)?;
        Ok(())
    }

    fn list_canonical_commits(&self) -> Result<Vec<CanonicalCommitRecord>, BlockPipelineError> {
        let mut records: Vec<CanonicalCommitRecord> = Vec::new();
        let keys = self
            .backend
            .list_keys(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE)
            .map_err(map_sqlite_commit_store_error)?;

        for key in keys {
            let key_height = parse_sqlite_canonical_commit_store_key(&key)?;
            let payload_bytes = self
                .backend
                .get(CANONICAL_COMMIT_SQLITE_ENTRIES_NAMESPACE, key.as_str())
                .map_err(map_sqlite_commit_store_error)?
                .ok_or_else(|| {
                    BlockPipelineError::CommitStore(format!(
                        "canonical commit sqlite row missing for key {key} (canonical_commit_store_sqlite_missing_entry)"
                    ))
                })?;
            let payload = String::from_utf8(payload_bytes).map_err(|_| {
                BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite payload is not utf-8 for key {key} (canonical_commit_store_sqlite_payload_not_utf8)"
                ))
            })?;
            if payload.trim().is_empty() {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite payload is empty for key {key} (canonical_commit_store_sqlite_payload_empty)"
                )));
            }
            let record = parse_canonical_commit_record(&payload)?;
            if record.block_height != key_height {
                return Err(BlockPipelineError::CommitStore(format!(
                    "canonical commit sqlite key height mismatch: key {}, payload {} (canonical_commit_store_sqlite_key_height_mismatch)",
                    key_height, record.block_height
                )));
            }
            if let Some(previous) = records.last() {
                if record.block_height <= previous.block_height {
                    return Err(BlockPipelineError::CommitStore(format!(
                        "canonical commit block height regression in persisted lineage: previous {}, found {} (canonical_commit_store_block_height_regression)",
                        previous.block_height, record.block_height
                    )));
                }
            }
            records.push(record);
        }
        Ok(records)
    }
}

fn sqlite_canonical_commit_store_key(block_height: u64) -> String {
    format!("height:{block_height:020}")
}

fn parse_sqlite_canonical_commit_store_key(key: &str) -> Result<u64, BlockPipelineError> {
    let Some(height_raw) = key.strip_prefix("height:") else {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        )));
    };
    if height_raw.len() != 20
        || !height_raw
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        )));
    }
    height_raw.parse::<u64>().map_err(|_| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite key is malformed: {key} (canonical_commit_store_sqlite_key_malformed)"
        ))
    })
}

fn map_sqlite_commit_store_error(error: SqliteStoreBackendError) -> BlockPipelineError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => BlockPipelineError::CommitStore(
            "canonical commit sqlite backend schema missing (canonical_commit_store_sqlite_backend_schema_missing)"
                .to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => BlockPipelineError::CommitStore(
            format!(
                "canonical commit sqlite backend schema invalid: {value} (canonical_commit_store_sqlite_backend_schema_invalid)"
            ),
        ),
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            BlockPipelineError::CommitStore(format!(
                "canonical commit sqlite backend schema mismatch: expected {expected}, found {found} (canonical_commit_store_sqlite_backend_schema_mismatch)"
            ))
        }
        SqliteStoreBackendError::InvalidPath => BlockPipelineError::CommitStore(
            "canonical commit sqlite path is invalid (canonical_commit_store_path_invalid)"
                .to_owned(),
        ),
        other => BlockPipelineError::CommitStore(format!(
            "canonical commit sqlite backend operation failed: {other} (canonical_commit_store_io)"
        )),
    }
}

fn serialize_canonical_commit_record(
    record: &CanonicalCommitRecord,
) -> Result<String, BlockPipelineError> {
    if record.block_height == 0 {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit block height must be positive (canonical_commit_store_block_height_invalid)"
                .to_owned(),
        ));
    }
    if record.transaction_ids.is_empty() {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit transaction ids cannot be empty (canonical_commit_store_transaction_ids_invalid)"
                .to_owned(),
        ));
    }
    validate_canonical_commit_store_field("payload_digest", record.payload_digest.as_str())?;
    let mut encoded_ids = Vec::with_capacity(record.transaction_ids.len());
    for tx_id in &record.transaction_ids {
        validate_canonical_commit_store_field("transaction_id", tx_id.as_str())?;
        if tx_id.contains(',') {
            return Err(BlockPipelineError::CommitStore(
                "canonical commit transaction id cannot contain ',' (canonical_commit_store_transaction_ids_invalid)"
                    .to_owned(),
            ));
        }
        encoded_ids.push(tx_id.as_str());
    }

    Ok(format!(
        "{}|{}|{}|{}\n",
        record.block_height,
        record.producer_role.as_str(),
        record.payload_digest,
        encoded_ids.join(",")
    ))
}

fn parse_canonical_commit_record(line: &str) -> Result<CanonicalCommitRecord, BlockPipelineError> {
    let mut segments = line.split('|');
    let block_height_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let producer_role_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let payload_digest = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    let transaction_ids_raw = segments.next().ok_or_else(|| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        ))
    })?;
    if segments.next().is_some() {
        return Err(BlockPipelineError::CommitStore(format!(
            "canonical commit record malformed: {line} (canonical_commit_store_record_malformed)"
        )));
    }

    let block_height = block_height_raw.parse::<u64>().map_err(|_| {
        BlockPipelineError::CommitStore(format!(
            "canonical commit block height is invalid: {block_height_raw} (canonical_commit_store_block_height_invalid)"
        ))
    })?;
    if block_height == 0 {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit block height must be positive (canonical_commit_store_block_height_invalid)"
                .to_owned(),
        ));
    }
    let producer_role = match producer_role_raw {
        "processor" => NodeRole::Processor,
        "listener" => NodeRole::Listener,
        "approver" => NodeRole::Approver,
        other => {
            return Err(BlockPipelineError::CommitStore(format!(
                "canonical commit producer role is invalid: {other} (canonical_commit_store_producer_role_invalid)"
            )));
        }
    };
    validate_canonical_commit_store_field("payload_digest", payload_digest)?;

    let mut seen_ids = BTreeSet::new();
    let mut transaction_ids = Vec::new();
    for tx_id in transaction_ids_raw.split(',').map(|value| value.trim()) {
        if tx_id.is_empty() {
            continue;
        }
        validate_canonical_commit_store_field("transaction_id", tx_id)?;
        if !seen_ids.insert(tx_id.to_owned()) {
            return Err(BlockPipelineError::CommitStore(format!(
                "canonical commit transaction id is duplicated: {tx_id} (canonical_commit_store_transaction_ids_invalid)"
            )));
        }
        transaction_ids.push(tx_id.to_owned());
    }
    if transaction_ids.is_empty() {
        return Err(BlockPipelineError::CommitStore(
            "canonical commit transaction ids cannot be empty (canonical_commit_store_transaction_ids_invalid)"
                .to_owned(),
        ));
    }

    Ok(CanonicalCommitRecord {
        block_height,
        producer_role,
        payload_digest: payload_digest.to_owned(),
        transaction_ids,
    })
}
