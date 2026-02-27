use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_workflow_contains_required_triggers() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    assert!(workflow.contains("push:"));
    assert!(workflow.contains("branches:"));
    assert!(workflow.contains("- main"));
    assert!(workflow.contains("workflow_dispatch:"));
    assert!(workflow.contains("schedule:"));
    assert!(workflow.contains("cron: '0 6 * * 1'"));
}

#[test]
fn spec_c02_to_c07_workflow_contains_required_lane_and_mode_markers() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    assert!(workflow.contains("e2e-sdk-direct:"));
    assert!(workflow.contains("e2e-mcp-agent:"));
    assert!(workflow.contains("e2e-cli-smoke:"));
    assert!(workflow.contains("--mode sdk-direct"));
    assert!(workflow.contains("--mode mcp-tau"));
    assert!(workflow.contains("--mode cli-scripted"));
}

#[test]
fn spec_c08_to_c14_workflow_enforces_external_live_execution_markers() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    assert!(workflow.contains("--enable-external-execution"));
    assert!(workflow.contains("KAMN_E2E_SDK_DIRECT_LIVE: \"1\""));
    assert!(workflow.contains("KAMN_E2E_CLI_SCRIPTED_LIVE: \"1\""));
    assert!(workflow.contains("KAMN_E2E_MCP_AGENT_LIVE: \"1\""));
    assert!(workflow.contains("KAMN_E2E_EXTERNAL_KAMN_PROCESSOR_BINARY"));
    assert!(workflow.contains("KAMN_E2E_EXTERNAL_KAMN_LISTENER_BINARY"));
    assert!(workflow.contains("KAMN_E2E_EXTERNAL_KAMN_APPROVER_BINARY"));
    assert!(workflow.contains("cargo build --release -p example-p2p"));
    assert!(workflow.contains("RUSTFLAGS=\"-C link-arg=-fuse-ld=bfd\""));
    assert!(workflow.contains("/tmp/kolme/target/release/example-p2p"));
    assert!(workflow.contains("api-server"));
    assert!(workflow.contains("--role processor"));
    assert!(workflow.contains("--role listener"));
    assert!(workflow.contains("--role approver"));
    assert!(workflow.contains("wait_for_port 127.0.0.1 3000"));
    assert!(workflow.contains("wait_for_http \"http://127.0.0.1:3000/healthz\""));
}

#[test]
fn spec_c10_phase4i_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase4i-gap-analysis.md"),
    )
    .expect("phase-4i docs marker artifact should exist");
    assert!(doc.contains("phase4i_status_before=partial"));
    assert!(doc.contains("phase4i_ci_live_lane_contract=implemented"));
    assert!(doc.contains("phase4i_status_after=implemented"));
}

#[test]
fn spec_c11_milestone_index_references_active_phase4i_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5580"));
}

#[test]
fn spec_c15_workflow_enforces_fail_closed_live_execution_gate_markers() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    assert!(workflow.contains("run_output_file=\"/tmp/e2e-live-run-output.json\""));
    assert!(workflow.contains("json.load(handle)"));
    assert!(workflow.contains("live execution overall_status must be PASS"));
    assert!(workflow.contains("scenario result status must be PASS"));
}

#[test]
fn spec_c16_workflow_enforces_evidence_verification_and_upload_markers() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    assert!(workflow.contains("Verify evidence bundle"));
    assert!(workflow.contains("--output /tmp/evidence/verification-report.json"));
    assert!(workflow.contains("Upload evidence"));
    assert!(workflow.contains("path: /tmp/evidence/"));
}

#[test]
fn spec_c17_workflow_applies_fail_closed_run_output_validation_for_all_modes() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");

    let run_output_marker_count = workflow
        .matches("run_output_file=\"/tmp/e2e-live-run-output.json\"")
        .count();
    let fail_closed_status_marker_count = workflow
        .matches("live execution overall_status must be PASS")
        .count();
    let fail_closed_scenario_marker_count = workflow
        .matches("scenario result status must be PASS")
        .count();

    assert_eq!(run_output_marker_count, 3);
    assert_eq!(fail_closed_status_marker_count, 3);
    assert_eq!(fail_closed_scenario_marker_count, 3);
}

#[test]
fn spec_c18_workflow_sets_explicit_tls_mode_for_all_live_lanes() {
    let root = repo_root();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");

    let tls_disable_marker_count = workflow.matches("KAMN_SERVICE_API_TLS_MODE=disable").count();
    assert_eq!(
        tls_disable_marker_count, 3,
        "workflow must set explicit disable mode for all three live lanes"
    );
}
