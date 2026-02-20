use kamn_core::{
    ChannelMetadata, ChannelRecordSnapshot, ChannelSnapshot, ChannelSnapshotStore,
    ChannelSnapshotStoreError, ChannelType, FileChannelSnapshotStore,
    FileMessageLifecycleSnapshotStore, FileTaskOperationSnapshotStore, MessageLifecycleSnapshot,
    MessageLifecycleSnapshotStore, MessageLifecycleSnapshotStoreError, MessageRecordSnapshot,
    MessageStatus, TaskOperationNoticeKind, TaskOperationRecordSnapshot, TaskOperationSnapshot,
    TaskOperationSnapshotStore, TaskOperationSnapshotStoreError, TaskState,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION, MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
    TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};
use std::ffi::OsString;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const FIXTURE: &str =
    include_str!("../../../fixtures/runtime/journal_wal_commit_boundary_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureMetadata {
    schema_version: String,
    journal_entry_shape: String,
    journal_entry_prefix: String,
    journal_entry_version: String,
    payload_encoding: String,
    commit_boundary_marker_taxonomy_version: String,
    commit_boundary_markers_csv: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureCase {
    case_id: String,
    store: String,
    snapshot_schema_version: u16,
    corrupt_tail_prefix: String,
    recovery_empty: String,
    recovery_clean: String,
    recovery_repaired_corrupt_payload: String,
}

#[derive(Debug)]
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        let sequence = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time must be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("kamn-{prefix}-{nanos}-{sequence}"));
        fs::create_dir_all(&path).expect("temp dir should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn parse_fixture() -> Result<(FixtureMetadata, Vec<FixtureCase>), String> {
    let mut schema_version = None;
    let mut journal_entry_shape = None;
    let mut journal_entry_prefix = None;
    let mut journal_entry_version = None;
    let mut payload_encoding = None;
    let mut commit_boundary_marker_taxonomy_version = None;
    let mut commit_boundary_markers_csv = None;
    let mut columns = None;
    let mut cases = Vec::new();

    for raw_line in FIXTURE.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.contains('=') {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("invalid metadata line: {line}"))?;
            let value = value.trim().to_owned();
            match key.trim() {
                "journal_wal_fixture_matrix_schema_version" => schema_version = Some(value),
                "journal_entry_shape" => journal_entry_shape = Some(value),
                "journal_entry_prefix" => journal_entry_prefix = Some(value),
                "journal_entry_version" => journal_entry_version = Some(value),
                "payload_encoding" => payload_encoding = Some(value),
                "commit_boundary_marker_taxonomy_version" => {
                    commit_boundary_marker_taxonomy_version = Some(value)
                }
                "commit_boundary_markers_csv" => commit_boundary_markers_csv = Some(value),
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }

        cases.push(parse_case_line(line)?);
    }

    let metadata = FixtureMetadata {
        schema_version: schema_version.ok_or("missing schema version metadata".to_owned())?,
        journal_entry_shape: journal_entry_shape
            .ok_or("missing journal entry shape metadata".to_owned())?,
        journal_entry_prefix: journal_entry_prefix
            .ok_or("missing journal entry prefix metadata".to_owned())?,
        journal_entry_version: journal_entry_version
            .ok_or("missing journal entry version metadata".to_owned())?,
        payload_encoding: payload_encoding.ok_or("missing payload encoding metadata".to_owned())?,
        commit_boundary_marker_taxonomy_version: commit_boundary_marker_taxonomy_version
            .ok_or("missing commit boundary taxonomy metadata".to_owned())?,
        commit_boundary_markers_csv: commit_boundary_markers_csv
            .ok_or("missing commit boundary markers csv metadata".to_owned())?,
        columns: columns.ok_or("missing columns metadata".to_owned())?,
    };

    if cases.is_empty() {
        return Err("fixture matrix must contain at least one case".to_owned());
    }

    Ok((metadata, cases))
}

fn parse_case_line(line: &str) -> Result<FixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 7 {
        return Err(format!(
            "expected 7 columns, found {} in '{line}'",
            parts.len()
        ));
    }

    let snapshot_schema_version = parts[2]
        .parse::<u16>()
        .map_err(|_| format!("snapshot_schema_version must be a u16 in '{line}'"))?;

    Ok(FixtureCase {
        case_id: parts[0].to_owned(),
        store: parts[1].to_owned(),
        snapshot_schema_version,
        corrupt_tail_prefix: parts[3].to_owned(),
        recovery_empty: parts[4].to_owned(),
        recovery_clean: parts[5].to_owned(),
        recovery_repaired_corrupt_payload: parts[6].to_owned(),
    })
}

fn did(id: &str) -> String {
    format!("kamn:did:agent:{id}")
}

fn channel_snapshot_fixture() -> ChannelSnapshot {
    ChannelSnapshot {
        schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
        records: vec![ChannelRecordSnapshot {
            channel_id: "channel-fixture-1".to_owned(),
            channel_type: ChannelType::Group,
            metadata: ChannelMetadata::Group,
            members: vec![did("alice"), did("bob")],
            admins: vec![did("alice")],
        }],
    }
}

fn message_snapshot_fixture() -> MessageLifecycleSnapshot {
    MessageLifecycleSnapshot {
        schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        records: vec![MessageRecordSnapshot {
            message_id: "message-fixture-1".to_owned(),
            sender: did("alice"),
            recipients: vec![did("bob")],
            created: "2026-01-01T00:00:00Z".to_owned(),
            expires: "2026-01-01T00:00:01Z".to_owned(),
            status: MessageStatus::Created,
            history: vec![MessageStatus::Created],
        }],
    }
}

fn task_snapshot_fixture() -> TaskOperationSnapshot {
    TaskOperationSnapshot {
        schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
        tasks: vec![TaskOperationRecordSnapshot {
            task_id: "task-fixture-1".to_owned(),
            requester: did("alice"),
            assignee: None,
            description: "fixture task".to_owned(),
            lifecycle_history: vec![TaskState::Submitted],
            dependencies: Vec::new(),
            notices: vec![TaskOperationNoticeKind::Submitted],
        }],
    }
}

fn journal_path_for(snapshot_path: &Path) -> PathBuf {
    let mut journal: OsString = snapshot_path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn assert_journal_schema(journal_path: &Path, expected_prefix: &str, expected_version: &str) {
    let payload = fs::read_to_string(journal_path).expect("journal file should be readable");
    let non_empty_lines: Vec<&str> = payload
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    assert_eq!(
        non_empty_lines.len(),
        1,
        "journal should contain exactly one deterministic entry line"
    );

    let parts: Vec<&str> = non_empty_lines[0].split('|').collect();
    assert_eq!(parts.len(), 3, "journal line should have 3 segments");
    assert_eq!(parts[0], expected_prefix, "journal prefix drift detected");
    assert_eq!(parts[1], expected_version, "journal version drift detected");

    let payload_hex = parts[2];
    assert!(
        !payload_hex.is_empty(),
        "journal payload hex segment must not be empty"
    );
    assert!(
        payload_hex.len().is_multiple_of(2),
        "journal payload hex must contain even number of characters"
    );
    assert!(
        payload_hex.chars().all(|ch| ch.is_ascii_hexdigit()),
        "journal payload must be valid hex"
    );
    assert!(
        payload_hex
            .chars()
            .all(|ch| !ch.is_ascii_uppercase() || !ch.is_ascii_hexdigit()),
        "journal payload hex should remain lowercase deterministic encoding"
    );
}

fn append_corrupt_tail_record(journal_path: &Path) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .expect("journal should be writable for corrupt-tail fixture");
    file.write_all(b"entry|1|deadbeefz\n")
        .expect("corrupt-tail payload should append");
}

fn run_channel_store_contract(case: &FixtureCase, metadata: &FixtureMetadata, tmp: &TempDir) {
    let snapshot_path = tmp.path().join("channel.snapshot");
    let journal_path = journal_path_for(&snapshot_path);

    let mut store = FileChannelSnapshotStore::new(snapshot_path.clone())
        .expect("channel file snapshot store should construct");

    let empty_recovery = store
        .recover_latest_and_repair()
        .expect("empty channel recovery should succeed");
    assert_eq!(empty_recovery.reason_code(), case.recovery_empty);

    store
        .write(channel_snapshot_fixture())
        .expect("channel snapshot write should succeed");
    assert_journal_schema(
        &journal_path,
        &metadata.journal_entry_prefix,
        &metadata.journal_entry_version,
    );

    let clean_recovery = store
        .recover_latest_and_repair()
        .expect("clean channel recovery should succeed");
    assert_eq!(clean_recovery.reason_code(), case.recovery_clean);

    append_corrupt_tail_record(&journal_path);
    match store.recover_latest_and_repair() {
        Err(ChannelSnapshotStoreError::InvalidPayload(value)) => {
            assert_eq!(value, format!("{}:2", case.corrupt_tail_prefix));
        }
        other => panic!("expected channel corrupt-tail invalid payload error, got {other:?}"),
    }

    let repaired_path = tmp.path().join("channel-repaired.snapshot");
    fs::write(&repaired_path, "malformed-channel-snapshot")
        .expect("repaired channel fixture should write invalid payload");
    let mut repaired_store = FileChannelSnapshotStore::new(repaired_path)
        .expect("channel repaired store should construct");
    let repaired = repaired_store
        .recover_latest_and_repair()
        .expect("channel repaired recovery should succeed");
    assert!(repaired.repaired, "channel repair flag should be true");
    assert_eq!(
        repaired.reason_code(),
        case.recovery_repaired_corrupt_payload
    );
}

fn run_message_store_contract(case: &FixtureCase, metadata: &FixtureMetadata, tmp: &TempDir) {
    let snapshot_path = tmp.path().join("message.snapshot");
    let journal_path = journal_path_for(&snapshot_path);

    let mut store = FileMessageLifecycleSnapshotStore::new(snapshot_path.clone())
        .expect("message file snapshot store should construct");

    let empty_recovery = store
        .recover_latest_and_repair()
        .expect("empty message recovery should succeed");
    assert_eq!(empty_recovery.reason_code(), case.recovery_empty);

    store
        .write(message_snapshot_fixture())
        .expect("message snapshot write should succeed");
    assert_journal_schema(
        &journal_path,
        &metadata.journal_entry_prefix,
        &metadata.journal_entry_version,
    );

    let clean_recovery = store
        .recover_latest_and_repair()
        .expect("clean message recovery should succeed");
    assert_eq!(clean_recovery.reason_code(), case.recovery_clean);

    append_corrupt_tail_record(&journal_path);
    match store.recover_latest_and_repair() {
        Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value)) => {
            assert_eq!(value, format!("{}:2", case.corrupt_tail_prefix));
        }
        other => panic!("expected message corrupt-tail invalid payload error, got {other:?}"),
    }

    let repaired_path = tmp.path().join("message-repaired.snapshot");
    fs::write(&repaired_path, "malformed-message-snapshot")
        .expect("repaired message fixture should write invalid payload");
    let mut repaired_store = FileMessageLifecycleSnapshotStore::new(repaired_path)
        .expect("message repaired store should construct");
    let repaired = repaired_store
        .recover_latest_and_repair()
        .expect("message repaired recovery should succeed");
    assert!(repaired.repaired, "message repair flag should be true");
    assert_eq!(
        repaired.reason_code(),
        case.recovery_repaired_corrupt_payload
    );
}

fn run_task_store_contract(case: &FixtureCase, metadata: &FixtureMetadata, tmp: &TempDir) {
    let snapshot_path = tmp.path().join("task.snapshot");
    let journal_path = journal_path_for(&snapshot_path);

    let mut store = FileTaskOperationSnapshotStore::new(snapshot_path.clone())
        .expect("task file snapshot store should construct");

    let empty_recovery = store
        .recover_latest_and_repair()
        .expect("empty task recovery should succeed");
    assert_eq!(empty_recovery.reason_code(), case.recovery_empty);

    store
        .write(task_snapshot_fixture())
        .expect("task snapshot write should succeed");
    assert_journal_schema(
        &journal_path,
        &metadata.journal_entry_prefix,
        &metadata.journal_entry_version,
    );

    let clean_recovery = store
        .recover_latest_and_repair()
        .expect("clean task recovery should succeed");
    assert_eq!(clean_recovery.reason_code(), case.recovery_clean);

    append_corrupt_tail_record(&journal_path);
    match store.recover_latest_and_repair() {
        Err(TaskOperationSnapshotStoreError::InvalidPayload(value)) => {
            assert_eq!(value, format!("{}:2", case.corrupt_tail_prefix));
        }
        other => panic!("expected task corrupt-tail invalid payload error, got {other:?}"),
    }

    let repaired_path = tmp.path().join("task-repaired.snapshot");
    fs::write(&repaired_path, "malformed-task-snapshot")
        .expect("repaired task fixture should write invalid payload");
    let mut repaired_store = FileTaskOperationSnapshotStore::new(repaired_path)
        .expect("task repaired store should construct");
    let repaired = repaired_store
        .recover_latest_and_repair()
        .expect("task repaired recovery should succeed");
    assert!(repaired.repaired, "task repair flag should be true");
    assert_eq!(
        repaired.reason_code(),
        case.recovery_repaired_corrupt_payload
    );
}

#[test]
fn unit_journal_wal_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 7 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_journal_wal_fixture_metadata_and_rows_are_deterministic() {
    let (metadata, cases) = parse_fixture().expect("fixture should parse");

    assert_eq!(
        metadata.journal_entry_shape, "entry|1|<payload_hex>",
        "journal entry schema marker drift detected"
    );
    assert_eq!(
        metadata.journal_entry_prefix, "entry",
        "journal entry prefix marker drift detected"
    );
    assert_eq!(
        metadata.journal_entry_version, "1",
        "journal entry version marker drift detected"
    );
    assert!(
        cases.iter().any(|case| case.store == "channel"),
        "fixture must include channel store row"
    );
    assert!(
        cases.iter().any(|case| case.store == "message_lifecycle"),
        "fixture must include message lifecycle store row"
    );
    assert!(
        cases.iter().any(|case| case.store == "task_operation"),
        "fixture must include task operation store row"
    );
}

#[test]
fn integration_journal_wal_store_contract_matches_fixture_markers() {
    let (metadata, cases) = parse_fixture().expect("fixture should parse");
    let temp = TempDir::new("journal-wal-contract");

    for case in cases {
        match case.store.as_str() {
            "channel" => {
                assert_eq!(
                    case.snapshot_schema_version,
                    CHANNEL_SNAPSHOT_SCHEMA_VERSION
                );
                run_channel_store_contract(&case, &metadata, &temp);
            }
            "message_lifecycle" => {
                assert_eq!(
                    case.snapshot_schema_version,
                    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION
                );
                run_message_store_contract(&case, &metadata, &temp);
            }
            "task_operation" => {
                assert_eq!(
                    case.snapshot_schema_version,
                    TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION
                );
                run_task_store_contract(&case, &metadata, &temp);
            }
            unknown => panic!("unknown fixture store row: {unknown}"),
        }
    }
}

#[test]
fn regression_journal_wal_marker_taxonomy_and_case_order_remain_stable() {
    let (metadata, cases) = parse_fixture().expect("fixture should parse");

    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.journal-wal-fixture-matrix.v1"
    );
    assert_eq!(metadata.payload_encoding, "lowercase_hex_utf8_payload");
    assert_eq!(
        metadata.commit_boundary_marker_taxonomy_version,
        "kamn.runtime.snapshot-journal-commit-boundary-markers.v1"
    );
    assert_eq!(
        metadata.commit_boundary_markers_csv,
        "channel_snapshot_recovery_empty,channel_snapshot_recovery_clean,channel_snapshot_recovery_repaired_corrupt_payload,message_lifecycle_snapshot_recovery_empty,message_lifecycle_snapshot_recovery_clean,message_lifecycle_snapshot_recovery_repaired_corrupt_payload,task_operation_snapshot_recovery_empty,task_operation_snapshot_recovery_clean,task_operation_snapshot_recovery_repaired_corrupt_payload"
    );
    assert_eq!(
        metadata.columns,
        "case_id|store|snapshot_schema_version|corrupt_tail_prefix|recovery_empty|recovery_clean|recovery_repaired_corrupt_payload"
    );

    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "channel_journal_contract",
            "message_lifecycle_journal_contract",
            "task_operation_journal_contract",
        ]
    );
}
