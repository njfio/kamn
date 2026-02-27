use std::path::{Path, PathBuf};

const JOURNAL_ENTRY_PREFIX: &str = "entry";
const JOURNAL_ENTRY_VERSION: &str = "1";

pub(crate) fn snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

pub(crate) fn encode_journal_hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

pub(crate) fn decode_journal_hex(value: &str) -> Option<Vec<u8>> {
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

pub(crate) fn parse_snapshot_journal_record(line: &str) -> Option<&str> {
    let mut parts = line.split('|');
    let prefix = parts.next()?;
    let version = parts.next()?;
    let payload_hex = parts.next()?;
    if prefix != JOURNAL_ENTRY_PREFIX
        || version != JOURNAL_ENTRY_VERSION
        || payload_hex.is_empty()
        || parts.next().is_some()
    {
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
    use super::{decode_journal_hex, encode_journal_hex, parse_snapshot_journal_record};

    #[test]
    fn unit_snapshot_journal_hex_roundtrip_contract() {
        let payload = b"snapshot|payload|v1";
        let encoded = encode_journal_hex(payload);
        let decoded = decode_journal_hex(encoded.as_str()).expect("encoded payload should decode");
        assert_eq!(decoded, payload);
    }

    #[test]
    fn unit_snapshot_journal_hex_rejects_odd_length_payload() {
        assert!(decode_journal_hex("abc").is_none());
    }

    #[test]
    fn unit_snapshot_journal_hex_rejects_non_hex_symbols() {
        assert!(decode_journal_hex("zz").is_none());
    }

    #[test]
    fn unit_snapshot_journal_record_parser_accepts_entry_v1() {
        let payload = parse_snapshot_journal_record("entry|1|616263")
            .expect("entry record should parse with payload");
        assert_eq!(payload, "616263");
    }

    #[test]
    fn unit_snapshot_journal_record_parser_rejects_invalid_shapes() {
        assert!(parse_snapshot_journal_record("").is_none());
        assert!(parse_snapshot_journal_record("entry|2|616263").is_none());
        assert!(parse_snapshot_journal_record("entry|1|").is_none());
        assert!(parse_snapshot_journal_record("entry|1|616263|extra").is_none());
    }
}
