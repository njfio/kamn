use kamn_snapshot_journal::{
    append_snapshot_journal_record, decode_snapshot_journal_hex, parse_snapshot_journal_record,
    parse_snapshot_journal_record_checked,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_journal_path() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be monotonic enough for tmp naming")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-snapshot-journal-roundtrip-{nanos}.journal"))
}

fn append_and_restore(payload: &str) -> String {
    let journal_path = temp_journal_path();
    append_snapshot_journal_record(&journal_path, payload).expect("append record");
    let journal = fs::read_to_string(&journal_path).expect("read journal");
    let line = journal.lines().next().expect("journal line");
    let payload_hex = parse_snapshot_journal_record_checked(line).expect("parse checked line");
    let decoded = decode_snapshot_journal_hex(&payload_hex).expect("decode payload hex");
    let restored = String::from_utf8(decoded).expect("utf8 payload");
    let _ = fs::remove_file(&journal_path);
    restored
}

#[test]
fn integration_append_and_parse_round_trip_whitespace_payload() {
    assert_eq!(append_and_restore(" \n\t "), " \n\t ");
}

#[test]
fn integration_append_and_parse_round_trip_unicode_payload() {
    let payload = "{\"message\":\"hello \\u{2603}\"}";
    assert_eq!(append_and_restore(payload), payload);
}

#[test]
fn integration_append_and_parse_round_trip_multiline_json_payload() {
    let payload = "{\n  \"state\": \"ok\",\n  \"count\": 2\n}";
    assert_eq!(append_and_restore(payload), payload);
}

#[test]
fn integration_checked_parse_and_decode_restore_payload_exactly() {
    let line = r#"{"schema_version":"kamn.snapshot-journal.entry.v1","payload_hex":"7b226b6579223a2276616c75655c6e6c696e65227d"}"#;
    let payload_hex =
        parse_snapshot_journal_record_checked(line).expect("json record should parse");
    assert_eq!(
        parse_snapshot_journal_record(line),
        Some(payload_hex.clone())
    );
    let decoded = decode_snapshot_journal_hex(&payload_hex).expect("decode payload hex");
    let restored = String::from_utf8(decoded).expect("utf8 payload");
    assert_eq!(restored, "{\"key\":\"value\\nline\"}");
}
