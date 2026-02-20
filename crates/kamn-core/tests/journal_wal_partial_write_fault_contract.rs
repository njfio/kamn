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
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const FIXTURE: &str =
    include_str!("../../../fixtures/runtime/journal_wal_partial_write_fault_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureMetadata {
    schema_version: String,
    reason_taxonomy_version: String,
    reason_codes_csv: String,
    columns: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureCase {
    case_id: String,
    store: String,
    fault_mode: String,
    expected_outcome: String,
    expected_marker: String,
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
    let mut reason_taxonomy_version = None;
    let mut reason_codes_csv = None;
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
                "journal_wal_partial_write_fixture_schema_version" => schema_version = Some(value),
                "journal_wal_partial_write_reason_taxonomy_version" => {
                    reason_taxonomy_version = Some(value)
                }
                "journal_wal_partial_write_reason_codes_csv" => reason_codes_csv = Some(value),
                "columns" => columns = Some(value),
                unknown => return Err(format!("unknown metadata key: {unknown}")),
            }
            continue;
        }

        cases.push(parse_case_line(line)?);
    }

    let metadata = FixtureMetadata {
        schema_version: schema_version.ok_or("missing schema version metadata".to_owned())?,
        reason_taxonomy_version: reason_taxonomy_version
            .ok_or("missing reason taxonomy metadata".to_owned())?,
        reason_codes_csv: reason_codes_csv.ok_or("missing reason codes csv metadata".to_owned())?,
        columns: columns.ok_or("missing columns metadata".to_owned())?,
    };

    if cases.is_empty() {
        return Err("fixture matrix must contain at least one case".to_owned());
    }

    Ok((metadata, cases))
}

fn parse_case_line(line: &str) -> Result<FixtureCase, String> {
    let parts: Vec<&str> = line.split('|').map(str::trim).collect();
    if parts.len() != 5 {
        return Err(format!(
            "expected 5 columns, found {} in '{line}'",
            parts.len()
        ));
    }

    Ok(FixtureCase {
        case_id: parts[0].to_owned(),
        store: parts[1].to_owned(),
        fault_mode: parts[2].to_owned(),
        expected_outcome: parts[3].to_owned(),
        expected_marker: parts[4].to_owned(),
    })
}

fn did(id: &str) -> String {
    format!("kamn:did:agent:{id}")
}

fn channel_snapshots() -> (ChannelSnapshot, ChannelSnapshot) {
    let first = ChannelSnapshot {
        schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
        records: vec![ChannelRecordSnapshot {
            channel_id: "channel-fixture-1".to_owned(),
            channel_type: ChannelType::Group,
            metadata: ChannelMetadata::Group,
            members: vec![did("owner"), did("member_1")],
            admins: vec![did("owner")],
        }],
    };
    let second = ChannelSnapshot {
        schema_version: CHANNEL_SNAPSHOT_SCHEMA_VERSION,
        records: vec![
            first.records[0].clone(),
            ChannelRecordSnapshot {
                channel_id: "channel-fixture-2".to_owned(),
                channel_type: ChannelType::Group,
                metadata: ChannelMetadata::Group,
                members: vec![did("owner"), did("member_2")],
                admins: vec![did("owner")],
            },
        ],
    };
    (first, second)
}

fn message_snapshots() -> (MessageLifecycleSnapshot, MessageLifecycleSnapshot) {
    let first = MessageLifecycleSnapshot {
        schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        records: vec![MessageRecordSnapshot {
            message_id: "message-fixture-1".to_owned(),
            sender: did("sender_1"),
            recipients: vec![did("recipient_1")],
            created: "2026-01-01T00:00:00Z".to_owned(),
            expires: "2026-01-01T00:00:01Z".to_owned(),
            status: MessageStatus::Created,
            history: vec![MessageStatus::Created],
        }],
    };
    let second = MessageLifecycleSnapshot {
        schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        records: vec![
            first.records[0].clone(),
            MessageRecordSnapshot {
                message_id: "message-fixture-2".to_owned(),
                sender: did("sender_2"),
                recipients: vec![did("recipient_2")],
                created: "2026-01-01T00:00:02Z".to_owned(),
                expires: "2026-01-01T00:00:03Z".to_owned(),
                status: MessageStatus::Created,
                history: vec![MessageStatus::Created],
            },
        ],
    };
    (first, second)
}

fn task_snapshots() -> (TaskOperationSnapshot, TaskOperationSnapshot) {
    let first = TaskOperationSnapshot {
        schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
        tasks: vec![TaskOperationRecordSnapshot {
            task_id: "task-fixture-1".to_owned(),
            requester: did("requester_1"),
            assignee: None,
            description: "first fixture task".to_owned(),
            lifecycle_history: vec![TaskState::Submitted],
            dependencies: Vec::new(),
            notices: vec![TaskOperationNoticeKind::Submitted],
        }],
    };
    let second = TaskOperationSnapshot {
        schema_version: TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
        tasks: vec![
            first.tasks[0].clone(),
            TaskOperationRecordSnapshot {
                task_id: "task-fixture-2".to_owned(),
                requester: did("requester_2"),
                assignee: None,
                description: "second fixture task".to_owned(),
                lifecycle_history: vec![TaskState::Submitted],
                dependencies: Vec::new(),
                notices: vec![TaskOperationNoticeKind::Submitted],
            },
        ],
    };
    (first, second)
}

fn journal_path_for(snapshot_path: &Path) -> PathBuf {
    let mut journal: OsString = snapshot_path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn append_partial_journal_tail(journal_path: &Path) {
    let mut file = OpenOptions::new()
        .append(true)
        .open(journal_path)
        .expect("journal should be writable for partial-tail fault injection");
    file.write_all(b"entry|1|abc\n")
        .expect("partial journal tail must be appended");
}

fn truncate_snapshot_file(path: &Path) {
    let payload = fs::read_to_string(path).expect("snapshot payload must exist before truncation");
    let truncated_len = (payload.len() / 2).max(1);
    let truncated = &payload[..truncated_len];
    fs::write(path, truncated).expect("snapshot payload truncation should succeed");
}

fn run_channel_case(case: &FixtureCase, tmp: &TempDir) {
    let snapshot_path = tmp
        .path()
        .join(format!("channel-{}.snapshot", case.case_id));
    let journal_path = journal_path_for(&snapshot_path);
    let _ = fs::remove_file(&snapshot_path);
    let _ = fs::remove_file(&journal_path);

    match case.fault_mode.as_str() {
        "partial_snapshot_file_write" => {
            assert_eq!(case.expected_outcome, "recovery_clean");
            let (first, second) = channel_snapshots();
            let mut store =
                FileChannelSnapshotStore::new(snapshot_path.clone()).expect("store should build");
            store.write(first).expect("first write should succeed");
            store
                .write(second.clone())
                .expect("second write should succeed");

            truncate_snapshot_file(&snapshot_path);
            assert_eq!(
                store
                    .read_latest()
                    .expect("journal should remain authoritative"),
                Some(second.clone())
            );

            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should stay clean with valid journal commit");
            assert!(!recovery.repaired);
            assert_eq!(recovery.reason_code(), case.expected_marker);
            assert_eq!(recovery.latest, Some(second));
        }
        "partial_journal_tail_write" => {
            assert_eq!(case.expected_outcome, "fail_closed_corrupt_tail");
            let (_, second) = channel_snapshots();
            let mut store =
                FileChannelSnapshotStore::new(snapshot_path.clone()).expect("store should build");
            store
                .write(second)
                .expect("snapshot write should create first journal entry");
            append_partial_journal_tail(&journal_path);
            match store.recover_latest_and_repair() {
                Err(ChannelSnapshotStoreError::InvalidPayload(value)) => {
                    assert_eq!(value, format!("{}:2", case.expected_marker));
                }
                other => panic!("expected channel corrupt-tail failure, got {other:?}"),
            }
        }
        "partial_snapshot_without_journal" => {
            assert_eq!(case.expected_outcome, "recovery_repaired_corrupt_payload");
            fs::write(&snapshot_path, "schema|1\nrecord|partial")
                .expect("invalid partial snapshot should be written");
            let mut store =
                FileChannelSnapshotStore::new(snapshot_path.clone()).expect("store should build");
            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should repair malformed payload when journal is absent");
            assert!(recovery.repaired);
            assert!(recovery.latest.is_none());
            assert_eq!(recovery.reason_code(), case.expected_marker);
        }
        unknown => panic!("unsupported channel fault mode: {unknown}"),
    }
}

fn run_message_case(case: &FixtureCase, tmp: &TempDir) {
    let snapshot_path = tmp
        .path()
        .join(format!("message-{}.snapshot", case.case_id));
    let journal_path = journal_path_for(&snapshot_path);
    let _ = fs::remove_file(&snapshot_path);
    let _ = fs::remove_file(&journal_path);

    match case.fault_mode.as_str() {
        "partial_snapshot_file_write" => {
            assert_eq!(case.expected_outcome, "recovery_clean");
            let (first, second) = message_snapshots();
            let mut store = FileMessageLifecycleSnapshotStore::new(snapshot_path.clone())
                .expect("store should build");
            store.write(first).expect("first write should succeed");
            store
                .write(second.clone())
                .expect("second write should succeed");

            truncate_snapshot_file(&snapshot_path);
            assert_eq!(
                store
                    .read_latest()
                    .expect("journal should remain authoritative"),
                Some(second.clone())
            );

            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should stay clean with valid journal commit");
            assert!(!recovery.repaired);
            assert_eq!(recovery.reason_code(), case.expected_marker);
            assert_eq!(recovery.latest, Some(second));
        }
        "partial_journal_tail_write" => {
            assert_eq!(case.expected_outcome, "fail_closed_corrupt_tail");
            let (_, second) = message_snapshots();
            let mut store = FileMessageLifecycleSnapshotStore::new(snapshot_path.clone())
                .expect("store should build");
            store
                .write(second)
                .expect("snapshot write should create first journal entry");
            append_partial_journal_tail(&journal_path);
            match store.recover_latest_and_repair() {
                Err(MessageLifecycleSnapshotStoreError::InvalidPayload(value)) => {
                    assert_eq!(value, format!("{}:2", case.expected_marker));
                }
                other => panic!("expected message corrupt-tail failure, got {other:?}"),
            }
        }
        "partial_snapshot_without_journal" => {
            assert_eq!(case.expected_outcome, "recovery_repaired_corrupt_payload");
            fs::write(&snapshot_path, "schema|1\nrecord|partial")
                .expect("invalid partial snapshot should be written");
            let mut store = FileMessageLifecycleSnapshotStore::new(snapshot_path.clone())
                .expect("store should build");
            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should repair malformed payload when journal is absent");
            assert!(recovery.repaired);
            assert!(recovery.latest.is_none());
            assert_eq!(recovery.reason_code(), case.expected_marker);
        }
        unknown => panic!("unsupported message fault mode: {unknown}"),
    }
}

fn run_task_case(case: &FixtureCase, tmp: &TempDir) {
    let snapshot_path = tmp.path().join(format!("task-{}.snapshot", case.case_id));
    let journal_path = journal_path_for(&snapshot_path);
    let _ = fs::remove_file(&snapshot_path);
    let _ = fs::remove_file(&journal_path);

    match case.fault_mode.as_str() {
        "partial_snapshot_file_write" => {
            assert_eq!(case.expected_outcome, "recovery_clean");
            let (first, second) = task_snapshots();
            let mut store = FileTaskOperationSnapshotStore::new(snapshot_path.clone())
                .expect("store should build");
            store.write(first).expect("first write should succeed");
            store
                .write(second.clone())
                .expect("second write should succeed");

            truncate_snapshot_file(&snapshot_path);
            assert_eq!(
                store
                    .read_latest()
                    .expect("journal should remain authoritative"),
                Some(second.clone())
            );

            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should stay clean with valid journal commit");
            assert!(!recovery.repaired);
            assert_eq!(recovery.reason_code(), case.expected_marker);
            assert_eq!(recovery.latest, Some(second));
        }
        "partial_journal_tail_write" => {
            assert_eq!(case.expected_outcome, "fail_closed_corrupt_tail");
            let (_, second) = task_snapshots();
            let mut store =
                FileTaskOperationSnapshotStore::new(snapshot_path.clone()).expect("store");
            store
                .write(second)
                .expect("snapshot write should create first journal entry");
            append_partial_journal_tail(&journal_path);
            match store.recover_latest_and_repair() {
                Err(TaskOperationSnapshotStoreError::InvalidPayload(value)) => {
                    assert_eq!(value, format!("{}:2", case.expected_marker));
                }
                other => panic!("expected task corrupt-tail failure, got {other:?}"),
            }
        }
        "partial_snapshot_without_journal" => {
            assert_eq!(case.expected_outcome, "recovery_repaired_corrupt_payload");
            fs::write(&snapshot_path, "schema|1\nrecord|partial")
                .expect("invalid partial snapshot should be written");
            let mut store =
                FileTaskOperationSnapshotStore::new(snapshot_path.clone()).expect("store");
            let recovery = store
                .recover_latest_and_repair()
                .expect("recovery should repair malformed payload when journal is absent");
            assert!(recovery.repaired);
            assert!(recovery.latest.is_none());
            assert_eq!(recovery.reason_code(), case.expected_marker);
        }
        unknown => panic!("unsupported task fault mode: {unknown}"),
    }
}

#[test]
fn unit_partial_write_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 5 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_partial_write_fixture_covers_required_fault_modes() {
    let (_, cases) = parse_fixture().expect("fixture should parse");

    for fault_mode in [
        "partial_snapshot_file_write",
        "partial_journal_tail_write",
        "partial_snapshot_without_journal",
    ] {
        assert!(
            cases.iter().any(|case| case.fault_mode == fault_mode),
            "fixture must include fault mode '{fault_mode}'"
        );
    }
    for store in ["channel", "message_lifecycle", "task_operation"] {
        assert!(
            cases.iter().any(|case| case.store == store),
            "fixture must include store row for '{store}'"
        );
    }
}

#[test]
fn integration_partial_write_fault_matrix_matches_store_contracts() {
    let (_, cases) = parse_fixture().expect("fixture should parse");
    let temp = TempDir::new("journal-wal-partial-write");

    for case in cases {
        match case.store.as_str() {
            "channel" => run_channel_case(&case, &temp),
            "message_lifecycle" => run_message_case(&case, &temp),
            "task_operation" => run_task_case(&case, &temp),
            unknown => panic!("unknown fixture store row: {unknown}"),
        }
    }
}

#[test]
fn regression_partial_write_taxonomy_and_matrix_rows_remain_stable() {
    let (metadata, cases) = parse_fixture().expect("fixture should parse");

    assert_eq!(
        metadata.schema_version,
        "kamn.runtime.journal-wal-partial-write-fault-matrix.v1"
    );
    assert_eq!(
        metadata.reason_taxonomy_version,
        "kamn.runtime.journal-wal-partial-write-reason-taxonomy.v1"
    );
    assert_eq!(
        metadata.reason_codes_csv,
        "partial_snapshot_file_write_recovered_from_journal,partial_journal_tail_write_fail_closed,partial_snapshot_without_journal_repaired"
    );
    assert_eq!(
        metadata.columns,
        "case_id|store|fault_mode|expected_outcome|expected_marker"
    );

    let observed_case_ids: Vec<String> = cases.into_iter().map(|case| case.case_id).collect();
    assert_eq!(
        observed_case_ids,
        vec![
            "channel_partial_snapshot_file_write",
            "message_partial_snapshot_file_write",
            "task_partial_snapshot_file_write",
            "channel_partial_journal_tail_write",
            "message_partial_journal_tail_write",
            "task_partial_journal_tail_write",
            "channel_partial_snapshot_without_journal",
            "message_partial_snapshot_without_journal",
            "task_partial_snapshot_without_journal",
        ]
    );
}

#[test]
fn performance_partial_write_fault_matrix_stays_within_ci_budget() {
    let (_, cases) = parse_fixture().expect("fixture should parse");
    let temp = TempDir::new("journal-wal-partial-write-perf");

    let started = Instant::now();
    for case in cases {
        match case.store.as_str() {
            "channel" => run_channel_case(&case, &temp),
            "message_lifecycle" => run_message_case(&case, &temp),
            "task_operation" => run_task_case(&case, &temp),
            unknown => panic!("unknown fixture store row: {unknown}"),
        }
    }
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 350,
        "partial-write fault matrix exceeded CI budget: {elapsed_millis}ms"
    );
}
