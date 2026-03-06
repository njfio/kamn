use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, parse_snapshot_journal_record,
    parse_snapshot_journal_record_checked, SnapshotJournalParseError,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_journal_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be monotonic enough for tmp naming")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-snapshot-journal-edge-cases-{nanos}.journal"))
}

#[test]
fn integration_append_multiple_records_preserves_order_and_newlines() {
    let journal_path = temp_journal_path();
    append_snapshot_journal_record(&journal_path, "alpha").expect("append first record");
    append_snapshot_journal_record(&journal_path, "beta").expect("append second record");

    let journal = fs::read_to_string(&journal_path).expect("read journal");
    let lines = journal.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);

    let first = parse_snapshot_journal_record(lines[0]).expect("parse first line");
    let second = parse_snapshot_journal_record(lines[1]).expect("parse second line");

    assert_eq!(
        String::from_utf8(decode_snapshot_journal_hex(&first).expect("decode first payload"))
            .expect("utf8 first payload"),
        "alpha"
    );
    assert_eq!(
        String::from_utf8(decode_snapshot_journal_hex(&second).expect("decode second payload"))
            .expect("utf8 second payload"),
        "beta"
    );
    assert!(journal.ends_with('\n'));

    let _ = fs::remove_file(&journal_path);
}

#[test]
fn integration_decode_accepts_uppercase_hex_payloads() {
    let decoded = decode_snapshot_journal_hex("414243").expect("uppercase hex should decode");
    assert_eq!(decoded, b"ABC");
}

#[test]
fn integration_checked_parse_accepts_corrupted_payload_hex_but_decode_fails_closed() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v1","payload_hex":"zz"}"#;
    let payload_hex =
        parse_snapshot_journal_record_checked(line).expect("json record should parse structurally");
    assert_eq!(payload_hex, "zz");
    assert!(decode_snapshot_journal_hex(&payload_hex).is_none());
    assert_eq!(parse_snapshot_journal_record(line), Some("zz".to_owned()));
}

#[test]
fn integration_checked_parse_rejects_missing_schema_version_field() {
    let line = r#"{"payload_hex":"616263"}"#;
    assert_eq!(
        parse_snapshot_journal_record_checked(line),
        Err(SnapshotJournalParseError::InvalidJson)
    );
    assert!(parse_snapshot_journal_record(line).is_none());
}

#[test]
fn integration_checked_parse_rejects_missing_payload_hex_field() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v1"}"#;
    assert_eq!(
        parse_snapshot_journal_record_checked(line),
        Err(SnapshotJournalParseError::InvalidJson)
    );
    assert!(parse_snapshot_journal_record(line).is_none());
}
