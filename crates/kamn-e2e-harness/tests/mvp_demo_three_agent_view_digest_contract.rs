use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::Path;

#[path = "support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;

#[allow(dead_code)]
#[path = "support/three_agent_view_artifacts.rs"]
mod three_agent_view_artifacts;

#[test]
fn spec_c01_command_rejects_stale_agent_a_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected(
        "agent-a-stale-view-digest",
        "agent-a-view.json",
        "agent-a-tampered",
        "agent_a_view_digest",
    );
}

#[test]
fn spec_c02_command_rejects_stale_agent_b_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected(
        "agent-b-stale-view-digest",
        "agent-b-view.json",
        "agent-b-tampered",
        "agent_b_view_digest",
    );
}

#[test]
fn spec_c03_command_rejects_stale_agent_c_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected(
        "agent-c-stale-view-digest",
        "agent-c-verifier-view.json",
        "agent-c-tampered",
        "agent_c_verifier_view_digest",
    );
}

fn assert_stale_view_digest_rejected(
    stem: &str,
    file_name: &str,
    marker: &str,
    expected_error: &str,
) {
    let root = three_agent_view_artifacts::temp_root(stem);
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(&root, None);
    append_json_marker(&root.join("proof").join(file_name), marker);
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );
    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("stale view digest must fail");
    assert!(err.contains(expected_error));
}

fn append_json_marker(path: &Path, marker: &str) {
    let raw = std::fs::read_to_string(path).expect("view fixture should be readable");
    let tampered = raw
        .strip_suffix('}')
        .map(|prefix| format!("{prefix},\"tamper_marker\":\"{marker}\"}}"))
        .expect("view fixture should be a JSON object");
    std::fs::write(path, tampered).expect("tampered view fixture should be written");
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
        agent_harness_evidence_path: None,
    }
}
