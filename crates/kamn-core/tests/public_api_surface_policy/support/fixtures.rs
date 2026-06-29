use crate::support::paths::fail;
use std::collections::BTreeMap;

pub(crate) fn parse_key_value_fixture(raw: &str, reason_code: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        store_fixture_entry(&mut map, line, index, reason_code);
    }
    map
}

pub(crate) fn required_value<'a>(
    map: &'a BTreeMap<String, String>,
    key: &str,
    reason_code: &str,
) -> &'a str {
    map.get(key)
        .map(String::as_str)
        .unwrap_or_else(|| fail(reason_code, &format!("missing required key {key}")))
}

pub(crate) fn required_i64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> i64 {
    let value = required_value(map, key, reason_code);
    value.parse::<i64>().unwrap_or_else(|error| {
        fail(
            reason_code,
            &format!("key {key} must parse as integer: {error}"),
        )
    })
}

fn store_fixture_entry(
    map: &mut BTreeMap<String, String>,
    line: &str,
    index: usize,
    reason_code: &str,
) {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return;
    }
    let (key, value) = trimmed.split_once('=').unwrap_or_else(|| {
        let line_number = index + 1;
        fail(
            reason_code,
            &format!("line {line_number} missing key=value form"),
        )
    });
    let key = key.trim();
    if key.is_empty() {
        let line_number = index + 1;
        fail(reason_code, &format!("line {line_number} has empty key"));
    }
    map.insert(key.to_owned(), value.trim().to_owned());
}
