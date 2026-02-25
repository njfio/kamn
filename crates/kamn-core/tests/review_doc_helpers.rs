#![allow(dead_code)]

use std::path::{Path, PathBuf};

pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

pub(crate) fn parse_marker_value(doc: &str, marker_key: &str) -> String {
    let needle = format!("{marker_key}=");
    let line = doc
        .lines()
        .find(|line| line.contains(needle.as_str()))
        .unwrap_or_else(|| panic!("missing marker {marker_key}"));
    line.split_once(needle.as_str())
        .unwrap_or_else(|| panic!("marker {marker_key} missing '=' separator"))
        .1
        .trim_matches('`')
        .trim()
        .to_string()
}

pub(crate) fn parse_marker_text(doc: &str, marker_key: &str) -> String {
    parse_marker_value(doc, marker_key)
}

pub(crate) fn parse_marker_usize(doc: &str, marker_key: &str) -> usize {
    let value = parse_marker_value(doc, marker_key);
    value
        .parse::<usize>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be an unsigned integer: {value}"))
}

pub(crate) fn parse_marker_f64(doc: &str, marker_key: &str) -> f64 {
    let value = parse_marker_value(doc, marker_key);
    value
        .parse::<f64>()
        .unwrap_or_else(|_| panic!("marker {marker_key} should be a float: {value}"))
}

pub(crate) fn parse_marker_csv(doc: &str, marker_key: &str) -> Vec<String> {
    parse_marker_value(doc, marker_key)
        .split(',')
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}
