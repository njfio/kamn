use crate::support::paths::fail;
use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) fn parse_key_value_fixture(raw: &str, reason_code: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for (index, line) in raw.lines().enumerate() {
        insert_fixture_entry(&mut map, line, index, reason_code);
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
        .unwrap_or_else(|| fail(reason_code, &format!("missing required key {}", key)))
}

pub(crate) fn required_i64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> i64 {
    required_value(map, key, reason_code)
        .parse::<i64>()
        .unwrap_or_else(|error| {
            fail(
                reason_code,
                &format!("key {} must parse as integer: {}", key, error),
            )
        })
}

pub(crate) fn required_f64(map: &BTreeMap<String, String>, key: &str, reason_code: &str) -> f64 {
    required_value(map, key, reason_code)
        .parse::<f64>()
        .unwrap_or_else(|error| {
            fail(
                reason_code,
                &format!("key {} must parse as float: {}", key, error),
            )
        })
}

pub(crate) fn optional_i64_with_default(
    map: &BTreeMap<String, String>,
    key: &str,
    default: i64,
    reason_code: &str,
) -> i64 {
    map.get(key)
        .map(|value| {
            value.parse::<i64>().unwrap_or_else(|error| {
                fail(
                    reason_code,
                    &format!("key {} must parse as integer: {}", key, error),
                )
            })
        })
        .unwrap_or(default)
}

pub(crate) fn optional_path(
    map: &BTreeMap<String, String>,
    key: &str,
    resolver: impl Fn(&str) -> PathBuf,
) -> Option<PathBuf> {
    map.get(key).and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
            None
        } else {
            Some(resolver(trimmed))
        }
    })
}

fn insert_fixture_entry(
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
        fail(
            reason_code,
            &format!("line {} missing key=value form", index + 1),
        )
    });
    let key = key.trim();
    if key.is_empty() {
        fail(reason_code, &format!("line {} has empty key", index + 1));
    }
    map.insert(key.to_owned(), value.trim().to_owned());
}
