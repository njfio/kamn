use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Runtime snapshot.
pub struct RuntimeSnapshot {
    state_version: u64,
    state_hash: String,
    cursor: u64,
}

impl RuntimeSnapshot {
    /// Handles new.
    pub fn new(state_version: u64, state_hash: &str) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(state_version, state_hash, state_version)
    }

    /// Handles with cursor.
    pub fn with_cursor(
        state_version: u64,
        state_hash: &str,
        cursor: u64,
    ) -> Result<Self, SnapshotRestoreError> {
        if state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if !is_valid_snapshot_hash(state_hash) {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        if cursor == 0 {
            return Err(SnapshotRestoreError::InvalidCursor);
        }
        Ok(Self {
            state_version,
            state_hash: state_hash.to_owned(),
            cursor,
        })
    }

    /// Handles state version.
    pub fn state_version(&self) -> u64 {
        self.state_version
    }

    /// Handles state hash.
    pub fn state_hash(&self) -> &str {
        &self.state_hash
    }

    /// Handles cursor.
    pub fn cursor(&self) -> u64 {
        self.cursor
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot restore error.
pub enum SnapshotRestoreError {
    /// Invalid state version.
    InvalidStateVersion,
    /// Invalid state hash.
    InvalidStateHash,
    /// Invalid cursor.
    InvalidCursor,
    /// State version mismatch.
    StateVersionMismatch {
        /// Expected state version.
        expected: u64,
        /// Observed state version.
        found: u64,
    },
    /// State hash mismatch.
    StateHashMismatch {
        /// Expected state hash.
        expected: String,
        /// Observed state hash.
        found: String,
    },
    /// Cursor mismatch.
    CursorMismatch {
        /// Expected cursor value.
        expected: u64,
        /// Observed cursor value.
        found: u64,
    },
}

impl Display for SnapshotRestoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidStateVersion => write!(f, "snapshot state version must be positive"),
            Self::InvalidStateHash => write!(f, "snapshot state hash cannot be empty"),
            Self::InvalidCursor => write!(f, "snapshot cursor must be positive"),
            Self::StateVersionMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot state version mismatch: expected {expected}, found {found}"
                )
            }
            Self::StateHashMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot state hash mismatch: expected {expected}, found {found}"
                )
            }
            Self::CursorMismatch { expected, found } => {
                write!(
                    f,
                    "snapshot cursor mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

impl Error for SnapshotRestoreError {}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot restore guard.
pub struct SnapshotRestoreGuard {
    expected_state_version: u64,
    expected_state_hash: String,
    expected_cursor: Option<u64>,
}

impl SnapshotRestoreGuard {
    /// Handles new.
    pub fn new(
        expected_state_version: u64,
        expected_state_hash: &str,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(expected_state_version, expected_state_hash, None)
    }

    /// Handles with expected cursor.
    pub fn with_expected_cursor(
        expected_state_version: u64,
        expected_state_hash: &str,
        expected_cursor: u64,
    ) -> Result<Self, SnapshotRestoreError> {
        Self::with_cursor(
            expected_state_version,
            expected_state_hash,
            Some(expected_cursor),
        )
    }

    fn with_cursor(
        expected_state_version: u64,
        expected_state_hash: &str,
        expected_cursor: Option<u64>,
    ) -> Result<Self, SnapshotRestoreError> {
        if expected_state_version == 0 {
            return Err(SnapshotRestoreError::InvalidStateVersion);
        }
        if !is_valid_snapshot_hash(expected_state_hash) {
            return Err(SnapshotRestoreError::InvalidStateHash);
        }
        if matches!(expected_cursor, Some(0)) {
            return Err(SnapshotRestoreError::InvalidCursor);
        }
        Ok(Self {
            expected_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            expected_cursor,
        })
    }

    /// Handles validate.
    pub fn validate(&self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotRestoreError> {
        if snapshot.state_version() != self.expected_state_version {
            return Err(SnapshotRestoreError::StateVersionMismatch {
                expected: self.expected_state_version,
                found: snapshot.state_version(),
            });
        }
        if snapshot.state_hash() != self.expected_state_hash {
            return Err(SnapshotRestoreError::StateHashMismatch {
                expected: self.expected_state_hash.clone(),
                found: snapshot.state_hash().to_owned(),
            });
        }
        if let Some(expected_cursor) = self.expected_cursor {
            if snapshot.cursor() != expected_cursor {
                return Err(SnapshotRestoreError::CursorMismatch {
                    expected: expected_cursor,
                    found: snapshot.cursor(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot store error.
pub enum SnapshotStoreError {
    /// Io.
    Io(String),
    /// Invalid payload.
    InvalidPayload(String),
    /// State version regression.
    StateVersionRegression {
        /// Previous.
        previous: u64,
        /// Found.
        found: u64,
    },
    /// Cursor regression.
    CursorRegression {
        /// Previous.
        previous: u64,
        /// Found.
        found: u64,
    },
    /// Stale state hash.
    StaleStateHash {
        /// State hash.
        state_hash: String,
        /// Previous version.
        previous_version: u64,
        /// Found version.
        found_version: u64,
    },
}

impl Display for SnapshotStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "snapshot store I/O error: {message}"),
            Self::InvalidPayload(payload) => {
                write!(f, "snapshot store invalid payload: {payload}")
            }
            Self::StateVersionRegression { previous, found } => {
                write!(
                    f,
                    "snapshot state version regression: previous {previous}, found {found}"
                )
            }
            Self::CursorRegression { previous, found } => {
                write!(
                    f,
                    "snapshot cursor regression: previous {previous}, found {found}"
                )
            }
            Self::StaleStateHash {
                state_hash,
                previous_version,
                found_version,
            } => {
                write!(
                    f,
                    "snapshot stale state hash detected for {state_hash}: versions {previous_version}->{found_version}"
                )
            }
        }
    }
}

impl Error for SnapshotStoreError {}

/// Runtime snapshot store.
pub trait RuntimeSnapshotStore {
    /// Handles write.
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError>;
    /// Handles read latest.
    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError>;
    /// Handles list.
    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError>;
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// In memory runtime snapshot store.
pub struct InMemoryRuntimeSnapshotStore {
    entries: Vec<RuntimeSnapshot>,
}

impl RuntimeSnapshotStore for InMemoryRuntimeSnapshotStore {
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError> {
        validate_snapshot_continuity(self.entries.last(), &snapshot)?;
        self.entries.push(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.entries.last().cloned())
    }

    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.entries.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// File runtime snapshot store.
pub struct FileRuntimeSnapshotStore {
    path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Snapshot recovery result.
pub struct SnapshotRecoveryResult {
    /// Latest.
    pub latest: Option<RuntimeSnapshot>,
    /// Recovered entries.
    pub recovered_entries: usize,
    /// Dropped corrupt entries.
    pub dropped_corrupt_entries: usize,
}

impl FileRuntimeSnapshotStore {
    /// Handles new.
    pub fn new(path: PathBuf) -> Result<Self, SnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(SnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        Ok(Self { path })
    }

    /// Handles recover latest and repair.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<SnapshotRecoveryResult, SnapshotStoreError> {
        if !self.path.exists() {
            return Ok(SnapshotRecoveryResult {
                latest: None,
                recovered_entries: 0,
                dropped_corrupt_entries: 0,
            });
        }

        let payload = fs::read_to_string(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let mut snapshots = Vec::new();
        let mut dropped_corrupt_entries = 0;
        let mut corruption_detected = false;

        for line in payload.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if corruption_detected {
                dropped_corrupt_entries += 1;
                continue;
            }

            match parse_snapshot_line(trimmed) {
                Ok(snapshot) => {
                    if validate_snapshot_continuity(snapshots.last(), &snapshot).is_err() {
                        corruption_detected = true;
                        dropped_corrupt_entries += 1;
                        continue;
                    }
                    snapshots.push(snapshot);
                }
                Err(_) => {
                    corruption_detected = true;
                    dropped_corrupt_entries += 1;
                }
            }
        }

        if corruption_detected {
            self.persist_snapshots(&snapshots)?;
        }

        Ok(SnapshotRecoveryResult {
            latest: snapshots.last().cloned(),
            recovered_entries: snapshots.len(),
            dropped_corrupt_entries,
        })
    }

    fn persist_snapshots(&self, snapshots: &[RuntimeSnapshot]) -> Result<(), SnapshotStoreError> {
        let mut serialized = String::new();
        for snapshot in snapshots {
            serialized.push_str(&format!(
                "{}|{}|{}\n",
                snapshot.state_version(),
                snapshot.state_hash(),
                snapshot.cursor()
            ));
        }
        fs::write(&self.path, serialized).map_err(|error| SnapshotStoreError::Io(error.to_string()))
    }
}

impl RuntimeSnapshotStore for FileRuntimeSnapshotStore {
    fn write(&mut self, snapshot: RuntimeSnapshot) -> Result<(), SnapshotStoreError> {
        if let Some(previous) = self.read_latest()? {
            validate_snapshot_continuity(Some(&previous), &snapshot)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let serialized = format!(
            "{}|{}|{}\n",
            snapshot.state_version(),
            snapshot.state_hash(),
            snapshot.cursor()
        );
        file.write_all(serialized.as_bytes())
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))
    }

    fn read_latest(&self) -> Result<Option<RuntimeSnapshot>, SnapshotStoreError> {
        Ok(self.list()?.pop())
    }

    fn list(&self) -> Result<Vec<RuntimeSnapshot>, SnapshotStoreError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let payload = fs::read_to_string(&self.path)
            .map_err(|error| SnapshotStoreError::Io(error.to_string()))?;
        let mut snapshots = Vec::new();

        for line in payload.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let snapshot = parse_snapshot_line(trimmed)?;
            validate_snapshot_continuity(snapshots.last(), &snapshot)?;
            snapshots.push(snapshot);
        }

        Ok(snapshots)
    }
}

fn parse_snapshot_line(line: &str) -> Result<RuntimeSnapshot, SnapshotStoreError> {
    let mut segments = line.split('|');
    let Some(state_version_raw) = segments.next() else {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    };
    let Some(state_hash_raw) = segments.next() else {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    };
    let cursor_raw = segments.next();
    if segments.next().is_some() {
        return Err(SnapshotStoreError::InvalidPayload(line.to_owned()));
    }

    let state_version = state_version_raw
        .parse::<u64>()
        .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))?;
    if let Some(cursor_raw) = cursor_raw {
        let cursor = cursor_raw
            .parse::<u64>()
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))?;
        RuntimeSnapshot::with_cursor(state_version, state_hash_raw, cursor)
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))
    } else {
        RuntimeSnapshot::new(state_version, state_hash_raw)
            .map_err(|_| SnapshotStoreError::InvalidPayload(line.to_owned()))
    }
}

fn is_valid_snapshot_hash(state_hash: &str) -> bool {
    !state_hash.trim().is_empty()
        && !state_hash.contains('|')
        && !state_hash.contains('\n')
        && !state_hash.contains('\r')
}

fn validate_snapshot_continuity(
    previous: Option<&RuntimeSnapshot>,
    snapshot: &RuntimeSnapshot,
) -> Result<(), SnapshotStoreError> {
    if let Some(previous) = previous {
        if snapshot.state_version() <= previous.state_version() {
            return Err(SnapshotStoreError::StateVersionRegression {
                previous: previous.state_version(),
                found: snapshot.state_version(),
            });
        }
        if snapshot.cursor() <= previous.cursor() {
            return Err(SnapshotStoreError::CursorRegression {
                previous: previous.cursor(),
                found: snapshot.cursor(),
            });
        }
        if snapshot.state_hash() == previous.state_hash() {
            return Err(SnapshotStoreError::StaleStateHash {
                state_hash: snapshot.state_hash().to_owned(),
                previous_version: previous.state_version(),
                found_version: snapshot.state_version(),
            });
        }
    }
    Ok(())
}
