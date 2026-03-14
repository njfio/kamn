use std::path::{Path, PathBuf};

pub(super) fn require_marker(document: &str, marker: &str, error: &str) -> Result<(), String> {
    if document.contains(marker) {
        return Ok(());
    }
    Err(error.to_owned())
}

pub(super) fn strip_json_whitespace(value: &str) -> String {
    value
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

pub(super) fn extract_json_string_marker(fragment: &str, marker: &str) -> Option<String> {
    let start = fragment.find(marker)?;
    let value_start = start + marker.len();
    let relative_end = fragment[value_start..].find('"')?;
    let value_end = value_start + relative_end;
    Some(fragment[value_start..value_end].to_owned())
}

pub(super) fn is_sha256_value(value: &str) -> bool {
    value
        .strip_prefix("sha256:")
        .is_some_and(|suffix| !suffix.is_empty())
}

fn is_timestamp_shape(bytes: &[u8]) -> bool {
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
}

fn parse_two_digits(bytes: &[u8], start: usize) -> Option<u8> {
    let tens = bytes.get(start)?;
    let ones = bytes.get(start + 1)?;
    if !tens.is_ascii_digit() || !ones.is_ascii_digit() {
        return None;
    }
    Some((tens - b'0') * 10 + (ones - b'0'))
}

fn parse_four_digits(bytes: &[u8], start: usize) -> Option<u16> {
    let thousands = bytes.get(start)?;
    let hundreds = bytes.get(start + 1)?;
    let tens = bytes.get(start + 2)?;
    let ones = bytes.get(start + 3)?;
    if !thousands.is_ascii_digit()
        || !hundreds.is_ascii_digit()
        || !tens.is_ascii_digit()
        || !ones.is_ascii_digit()
    {
        return None;
    }
    Some(
        (thousands - b'0') as u16 * 1000
            + (hundreds - b'0') as u16 * 100
            + (tens - b'0') as u16 * 10
            + (ones - b'0') as u16,
    )
}

fn parse_timestamp_fields(bytes: &[u8]) -> Option<(u16, u8, u8, u8, u8, u8)> {
    Some((
        parse_four_digits(bytes, 0)?,
        parse_two_digits(bytes, 5)?,
        parse_two_digits(bytes, 8)?,
        parse_two_digits(bytes, 11)?,
        parse_two_digits(bytes, 14)?,
        parse_two_digits(bytes, 17)?,
    ))
}

fn is_timestamp_value_valid(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> bool {
    year > 0
        && (1..=12).contains(&month)
        && (1..=31).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 59
}

pub(super) fn is_rfc3339_utc_z_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if !is_timestamp_shape(bytes) {
        return false;
    }
    let Some((year, month, day, hour, minute, second)) = parse_timestamp_fields(bytes) else {
        return false;
    };
    is_timestamp_value_valid(year, month, day, hour, minute, second)
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<std::fs::DirEntry>, String> {
    let mut entries = std::fs::read_dir(dir)
        .map_err(|error| {
            format!(
                "failed to read evidence directory {}: {error}",
                dir.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to read evidence directory {}: {error}",
                dir.display()
            )
        })?;
    entries.sort_by_key(|entry| entry.path());
    Ok(entries)
}

pub(super) fn collect_evidence_json_artifacts(
    dir: &Path,
    artifacts: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in sorted_dir_entries(dir)? {
        let path = entry.path();
        if path.is_dir() {
            collect_evidence_json_artifacts(path.as_path(), artifacts)?;
            continue;
        }
        let is_json = path.extension().and_then(|value| value.to_str()) == Some("json");
        if is_json {
            artifacts.push(path);
        }
    }
    Ok(())
}
