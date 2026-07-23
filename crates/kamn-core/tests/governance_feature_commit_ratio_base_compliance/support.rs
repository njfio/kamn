use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_GOVERNANCE_RATIO: &str = "0.20";

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("repo root parent")
        .to_path_buf()
}

pub fn temp_path(name: &str, extension: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{stamp}.{extension}"))
}

pub fn output_json(name: &str) -> PathBuf {
    temp_path(name, "json")
}

pub fn subjects_file(name: &str, subjects: &[&str]) -> PathBuf {
    let path = temp_path(name, "txt");
    let mut body = subjects.join("\n");
    body.push('\n');
    std::fs::write(&path, body).expect("subjects file should be written");
    path
}

pub fn run_subject_checker(
    name: &str,
    subjects: &[&str],
    window_size: &str,
    max_ratio: &str,
) -> (Output, Value) {
    let subject_path = subjects_file(name, subjects);
    let report_path = output_json(name);
    let output = Command::new("python3")
        .arg("scripts/ci/check_governance_feature_commit_ratio.py")
        .arg("--commit-subjects-file")
        .arg(&subject_path)
        .arg("--window-size")
        .arg(window_size)
        .arg("--max-governance-ratio")
        .arg(max_ratio)
        .arg("--output-json")
        .arg(&report_path)
        .current_dir(repo_root())
        .output()
        .expect("subject checker should launch");
    (output, read_report(&report_path))
}

pub fn read_report(path: &PathBuf) -> Value {
    let raw = std::fs::read_to_string(path).expect("report should exist");
    serde_json::from_str(&raw).expect("report should be valid json")
}

pub fn status(report: &Value) -> &str {
    report["status"].as_str().expect("status should be string")
}

pub fn string_field(report: &Value, field: &str) -> String {
    report[field]
        .as_str()
        .unwrap_or_else(|| panic!("{field} should be string"))
        .to_owned()
}

pub fn u64_field(report: &Value, field: &str) -> u64 {
    report[field]
        .as_u64()
        .unwrap_or_else(|| panic!("{field} should be u64"))
}

pub fn f64_field(report: &Value, field: &str) -> f64 {
    report[field]
        .as_f64()
        .unwrap_or_else(|| panic!("{field} should be f64"))
}
