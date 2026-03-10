impl ChannelSnapshotStore for InMemoryChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        self.latest = Some(snapshot);
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        Ok(self.latest.clone())
    }
}

impl ChannelRecoveryResult {
    /// Returns the deterministic recovery reason code.
    pub fn reason_code(&self) -> &'static str {
        self.reason_code
    }
}

impl FileChannelSnapshotStore {
    /// Create a file-backed store for the given snapshot path.
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        if path.as_os_str().is_empty() {
            return Err(ChannelSnapshotStoreError::InvalidPayload(
                "snapshot file path cannot be empty".to_owned(),
            ));
        }
        let journal_path = channel_snapshot_journal_path(&path);
        Ok(Self { path, journal_path })
    }

    /// Attempt to read latest snapshot and repair invalid persisted payloads.
    pub fn recover_latest_and_repair(
        &mut self,
    ) -> Result<ChannelRecoveryResult, ChannelSnapshotStoreError> {
        if !self.path.exists() && !self.journal_path.exists() {
            return Ok(ChannelRecoveryResult {
                latest: None,
                repaired: false,
                reason_code: "channel_snapshot_recovery_empty",
            });
        }

        match self.read_latest() {
            Ok(snapshot) => Ok(ChannelRecoveryResult {
                latest: snapshot,
                repaired: false,
                reason_code: "channel_snapshot_recovery_clean",
            }),
            Err(ChannelSnapshotStoreError::InvalidPayload(value))
                if value.starts_with(CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX) =>
            {
                Err(ChannelSnapshotStoreError::InvalidPayload(value))
            }
            Err(ChannelSnapshotStoreError::InvalidPayload(_))
            | Err(ChannelSnapshotStoreError::Snapshot(_)) => {
                fs::write(&self.path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                fs::write(&self.journal_path, "")
                    .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
                Ok(ChannelRecoveryResult {
                    latest: None,
                    repaired: true,
                    reason_code: "channel_snapshot_recovery_repaired_corrupt_payload",
                })
            }
            Err(error) => Err(error),
        }
    }
}

impl ChannelSnapshotStore for FileChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        file.write_all(payload.as_bytes())
            .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
        append_channel_snapshot_journal_record(&self.journal_path, &payload)
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let journal_snapshot = replay_channel_snapshot_journal(&self.journal_path)?;
        if journal_snapshot.is_some() {
            return Ok(journal_snapshot);
        }
        read_channel_snapshot_file(&self.path)
    }
}

impl SqliteChannelSnapshotStore {
    /// Creates a sqlite-backed channel snapshot store rooted at `path`.
    pub fn new(path: PathBuf) -> Result<Self, ChannelSnapshotStoreError> {
        let backend = SqliteStoreBackend::open(path.as_path()).map_err(map_sqlite_store_error)?;
        Ok(Self { backend })
    }
}

impl ChannelSnapshotStore for SqliteChannelSnapshotStore {
    fn write(&mut self, snapshot: ChannelSnapshot) -> Result<(), ChannelSnapshotStoreError> {
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        let payload = serialize_channel_snapshot(&snapshot)?;
        self.backend
            .put("channel_snapshot_store", "latest", payload.as_bytes())
            .map_err(map_sqlite_store_error)?;
        Ok(())
    }

    fn read_latest(&self) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
        let Some(payload_bytes) = self
            .backend
            .get("channel_snapshot_store", "latest")
            .map_err(map_sqlite_store_error)?
        else {
            return Ok(None);
        };
        let payload = String::from_utf8(payload_bytes).map_err(|_| {
            ChannelSnapshotStoreError::InvalidPayload(
                "channel snapshot sqlite payload is not utf-8".to_owned(),
            )
        })?;
        if payload.trim().is_empty() {
            return Ok(None);
        }
        let snapshot = parse_channel_snapshot_payload(&payload)?;
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(ChannelSnapshotStoreError::Snapshot)?;
        Ok(Some(snapshot))
    }
}

fn map_sqlite_store_error(error: SqliteStoreBackendError) -> ChannelSnapshotStoreError {
    match error {
        SqliteStoreBackendError::SchemaVersionMissing => ChannelSnapshotStoreError::InvalidPayload(
            "channel snapshot sqlite schema missing".to_owned(),
        ),
        SqliteStoreBackendError::SchemaVersionInvalid(value) => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema invalid: {value}"
            ))
        }
        SqliteStoreBackendError::SchemaVersionMismatch { expected, found } => {
            ChannelSnapshotStoreError::InvalidPayload(format!(
                "channel snapshot sqlite schema mismatch: expected {expected}, found {found}"
            ))
        }
        SqliteStoreBackendError::InvalidPath => ChannelSnapshotStoreError::InvalidPayload(
            "snapshot file path cannot be empty".to_owned(),
        ),
        other => ChannelSnapshotStoreError::Io(other.to_string()),
    }
}

const CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX: &str = "channel_snapshot_journal_corrupt_tail";

fn channel_snapshot_journal_path(path: &Path) -> PathBuf {
    default_snapshot_journal_path(path)
}

fn read_channel_snapshot_file(
    path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    if payload.trim().is_empty() {
        return Ok(None);
    }
    let snapshot = parse_channel_snapshot_payload(&payload)?;
    let mut verifier = ChannelStore::new();
    verifier
        .restore_snapshot(snapshot.clone())
        .map_err(ChannelSnapshotStoreError::Snapshot)?;
    Ok(Some(snapshot))
}

fn append_channel_snapshot_journal_record(
    journal_path: &Path,
    payload: &str,
) -> Result<(), ChannelSnapshotStoreError> {
    append_snapshot_journal_record(journal_path, payload)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    Ok(())
}

fn replay_channel_snapshot_journal(
    journal_path: &Path,
) -> Result<Option<ChannelSnapshot>, ChannelSnapshotStoreError> {
    if !journal_path.exists() {
        return Ok(None);
    }

    let payload = fs::read_to_string(journal_path)
        .map_err(|error| ChannelSnapshotStoreError::Io(error.to_string()))?;
    let mut latest = None;

    for (index, line) in payload.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let payload_hex = parse_channel_snapshot_journal_record(trimmed, index + 1)?;
        let payload_bytes = decode_snapshot_journal_hex(&payload_hex)
            .ok_or_else(|| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let payload = String::from_utf8(payload_bytes)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let snapshot = parse_channel_snapshot_payload(&payload)
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        let mut verifier = ChannelStore::new();
        verifier
            .restore_snapshot(snapshot.clone())
            .map_err(|_| channel_snapshot_journal_corrupt_tail(index + 1))?;
        latest = Some(snapshot);
    }

    Ok(latest)
}

fn parse_channel_snapshot_journal_record(
    line: &str,
    index: usize,
) -> Result<String, ChannelSnapshotStoreError> {
    parse_snapshot_journal_record(line).ok_or_else(|| channel_snapshot_journal_corrupt_tail(index))
}

fn channel_snapshot_journal_corrupt_tail(index: usize) -> ChannelSnapshotStoreError {
    ChannelSnapshotStoreError::InvalidPayload(format!(
        "{CHANNEL_SNAPSHOT_JOURNAL_CORRUPT_TAIL_PREFIX}:{index}"
    ))
}
