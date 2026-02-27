#![warn(missing_docs)]
//! Shared snapshot-journal helpers extracted from `kamn-core`.

use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Error, ErrorKind, Write};
use std::path::{Path, PathBuf};

const SNAPSHOT_JOURNAL_ENTRY_SCHEMA_VERSION: &str = "kamn.snapshot-journal.entry.v1";
const HEX: &[u8; 16] = b"0123456789abcdef";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SnapshotJournalRecord {
    schema_version: String,
    payload_hex: String,
}

/// Returns the deterministic `<snapshot>.journal` sidecar path.
pub fn default_snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

/// Appends one snapshot payload record as a JSON line.
pub fn append_snapshot_journal_record(journal_path: &Path, payload: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(journal_path)?;
    let record = SnapshotJournalRecord {
        schema_version: SNAPSHOT_JOURNAL_ENTRY_SCHEMA_VERSION.to_owned(),
        payload_hex: encode_snapshot_journal_hex(payload.as_bytes()),
    };
    let encoded = serde_json::to_string(&record)
        .map_err(|error| Error::new(ErrorKind::InvalidData, error.to_string()))?;
    file.write_all(encoded.as_bytes())?;
    file.write_all(b"\n")
}

/// Parses one snapshot-journal JSON line and returns its `payload_hex` value.
pub fn parse_snapshot_journal_record(line: &str) -> Option<String> {
    let record: SnapshotJournalRecord = serde_json::from_str(line).ok()?;
    if record.schema_version != SNAPSHOT_JOURNAL_ENTRY_SCHEMA_VERSION
        || record.payload_hex.is_empty()
    {
        return None;
    }
    Some(record.payload_hex)
}

/// Decodes lowercase/uppercase hex payload strings.
pub fn decode_snapshot_journal_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index < bytes.len() {
        let high = decode_snapshot_journal_nibble(bytes[index])?;
        let low = decode_snapshot_journal_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Some(decoded)
}

fn encode_snapshot_journal_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn decode_snapshot_journal_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        append_snapshot_journal_record, decode_snapshot_journal_hex, parse_snapshot_journal_record,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_journal_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be monotonic enough for tmp naming")
            .as_nanos();
        std::env::temp_dir().join(format!("kamn-snapshot-journal-{nanos}.journal"))
    }

    #[test]
    fn unit_snapshot_journal_json_line_roundtrip_preserves_payload() {
        let journal_path = temp_journal_path();
        let payload = r#"{"schema":"v1","state":"ok"}"#;

        append_snapshot_journal_record(&journal_path, payload).expect("append record");
        let lines = fs::read_to_string(&journal_path).expect("read journal");
        let line = lines.lines().next().expect("journal line");
        let payload_hex = parse_snapshot_journal_record(line).expect("parse json journal line");
        let decoded = decode_snapshot_journal_hex(&payload_hex).expect("decode payload hex");
        let restored = String::from_utf8(decoded).expect("utf8 payload");
        assert_eq!(restored, payload);

        let _ = fs::remove_file(&journal_path);
    }

    #[test]
    fn regression_issue_6205_pipe_delimited_record_is_rejected_after_json_migration() {
        let legacy = "entry|1|616263";
        assert!(parse_snapshot_journal_record(legacy).is_none());
    }
}
