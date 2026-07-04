use std::collections::BTreeMap;

use super::source_path_support::fail;

pub fn parse_key_value_fixture(raw: &str, reason_code: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        insert_fixture_line(&mut map, line, index, reason_code);
    }
    map
}

pub fn required_value<'a>(
    map: &'a BTreeMap<String, String>,
    key: &str,
    reason_code: &str,
) -> &'a str {
    map.get(key)
        .map(String::as_str)
        .unwrap_or_else(|| fail(reason_code, &format!("missing required key {key}")))
}

pub fn required_i64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> i64 {
    required_value(map, key, reason_code)
        .parse::<i64>()
        .unwrap_or_else(|error| invalid_integer(key, error, reason_code))
}

fn insert_fixture_line(
    map: &mut BTreeMap<String, String>,
    line: &str,
    index: usize,
    reason_code: &str,
) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return;
    }
    let (key, value) = parse_fixture_pair(trimmed, index, reason_code);
    map.insert(key.to_owned(), value.to_owned());
}

fn parse_fixture_pair<'a>(trimmed: &'a str, index: usize, reason_code: &str) -> (&'a str, &'a str) {
    let line_number = index + 1;
    let (key, value) = trimmed.split_once('=').unwrap_or_else(|| {
        fail(
            reason_code,
            &format!("line {line_number} missing key=value form"),
        )
    });
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        fail(reason_code, &format!("line {line_number} has empty key"));
    }
    (key, value)
}

fn invalid_integer(key: &str, error: std::num::ParseIntError, reason_code: &str) -> ! {
    fail(
        reason_code,
        &format!("key {key} must parse as integer: {error}"),
    )
}
