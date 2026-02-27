use std::path::{Path, PathBuf};

/// Deterministically derives the companion `.journal` path for a snapshot file.
pub fn snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

/// Encodes bytes as lowercase hex for journal line payloads.
pub fn encode_journal_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

/// Decodes lowercase/uppercase hex into bytes.
pub fn decode_journal_hex(value: &str) -> Option<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        return None;
    }
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len() / 2);
    let mut index = 0usize;
    while index < bytes.len() {
        let high = decode_journal_nibble(bytes[index])?;
        let low = decode_journal_nibble(bytes[index + 1])?;
        decoded.push((high << 4) | low);
        index += 2;
    }
    Some(decoded)
}

/// Parses an `entry|1|<payload-hex>` journal line and returns the payload segment.
pub fn parse_snapshot_journal_record(line: &str) -> Option<&str> {
    let mut parts = line.split('|');
    let prefix = parts.next()?;
    let version = parts.next()?;
    let payload_hex = parts.next()?;
    if prefix != "entry" || version != "1" || payload_hex.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(payload_hex)
}

fn decode_journal_nibble(value: u8) -> Option<u8> {
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
        decode_journal_hex, encode_journal_hex, parse_snapshot_journal_record,
        snapshot_journal_path,
    };
    use std::path::Path;

    #[test]
    fn snapshot_journal_path_appends_suffix() {
        let path = Path::new("/tmp/kamn.snapshot");
        assert_eq!(
            snapshot_journal_path(path).to_string_lossy(),
            "/tmp/kamn.snapshot.journal"
        );
    }

    #[test]
    fn journal_hex_roundtrip() {
        let payload = b"snapshot payload v1";
        let encoded = encode_journal_hex(payload);
        let decoded = decode_journal_hex(encoded.as_str()).expect("hex should decode");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn parse_snapshot_journal_record_accepts_expected_shape() {
        let line = "entry|1|deadbeef";
        assert_eq!(parse_snapshot_journal_record(line), Some("deadbeef"));
    }

    #[test]
    fn parse_snapshot_journal_record_rejects_invalid_shape() {
        assert!(parse_snapshot_journal_record("entry|2|deadbeef").is_none());
        assert!(parse_snapshot_journal_record("entry|1|").is_none());
        assert!(parse_snapshot_journal_record("entry|1|deadbeef|extra").is_none());
    }
}
