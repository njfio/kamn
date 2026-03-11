use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub const BASE_SHA: &str = "d2c2fe1b901a1d53ea419f31778e1d836f2b1323";
pub const WINDOW_SIZE: &str = "50";
pub const MAX_GOVERNANCE_RATIO: &str = "0.20";

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir parent")
        .parent()
        .expect("repo root parent")
        .to_path_buf()
}

pub fn output_json(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{name}-{stamp}.json"))
}

pub fn run_checker(head_sha: &str, output_json: &PathBuf) -> std::process::Output {
    Command::new("python3")
        .arg("scripts/ci/check_governance_feature_commit_ratio.py")
        .arg("--repo-root")
        .arg(repo_root())
        .arg("--base-sha")
        .arg(BASE_SHA)
        .arg("--head-sha")
        .arg(head_sha)
        .arg("--window-size")
        .arg(WINDOW_SIZE)
        .arg("--max-governance-ratio")
        .arg(MAX_GOVERNANCE_RATIO)
        .arg("--output-json")
        .arg(output_json)
        .current_dir(repo_root())
        .output()
        .expect("checker should launch")
}

pub fn read_report(path: &PathBuf) -> Value {
    let raw = std::fs::read_to_string(path).expect("report should exist");
    serde_json::from_str(&raw).expect("report should be valid json")
}

pub fn status(report: &Value) -> &str {
    report["status"].as_str().expect("status should be string")
}
