#[path = "mvp_demo_agent_harness_claim_contract/canonical_observation_receipts_contract.rs"]
mod canonical_observation_receipts_contract;
#[path = "mvp_demo_agent_harness_claim_contract/support.rs"]
mod support;
#[path = "mvp_demo_agent_harness_claim_contract/three_agent_actor_receipts_contract.rs"]
mod three_agent_actor_receipts_contract;
#[path = "mvp_demo_agent_harness_claim_contract/three_agent_actor_rehearsal_contract.rs"]
mod three_agent_actor_rehearsal_contract;
#[path = "mvp_demo_agent_harness_claim_contract/three_agent_boundary_contract.rs"]
mod three_agent_boundary_contract;

use kamn_e2e_harness::{execute_mvp_demo_contract, execute_verify_mvp_demo_contract};
use std::path::Path;
use std::process::Command;
use support::*;

#[test]
fn spec_c01_direct_report_without_agent_harness_still_passes() {
    let root = temp_root("direct");
    let report = write_report(&root, direct_report(&root));

    execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("direct MVP report should not require agent harness proof");
}

#[test]
fn spec_c02_command_rejects_agent_harness_claim_without_artifact() {
    let root = temp_root("missing-artifact");
    let report = write_report(&root, report_with_agent_claim(&root, None));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("claimed agent harness proof must name an artifact");

    assert!(err.contains("agent harness evidence"));
}

#[test]
fn spec_c03_command_rejects_invalid_agent_harness_artifacts() {
    for (stem, private_visible, settlement_label, expected) in [
        (
            "private-leak",
            true,
            "devnet-backed",
            "verifier_private_view_visible",
        ),
        (
            "local-settlement",
            false,
            "local-only",
            "settlement_claim_label",
        ),
    ] {
        let root = temp_root(stem);
        let artifact = write_artifact(
            &root,
            agent_artifact(&root, private_visible, settlement_label),
        );
        let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

        let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
            .expect_err("invalid agent harness evidence must fail");
        assert!(err.contains(expected));
    }
}

#[test]
fn spec_c04_command_accepts_valid_agent_harness_artifact() {
    let root = temp_root("valid");
    let artifact = write_artifact(&root, agent_artifact(&root, false, "devnet-backed"));
    let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

    execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("valid MCP-agent harness evidence should verify");
}

#[test]
fn spec_c05_command_accepts_pi_extension_tool_surface() {
    let root = temp_root("pi-surface");
    let artifact = write_latest_artifact(
        &root,
        agent_artifact_with_surface(&root, false, "devnet-backed", "pi-extension-tools"),
    );
    let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

    execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("Pi extension tool evidence should verify");
}

#[test]
fn spec_c06_demo_mvp_can_include_agent_harness_evidence() {
    let root = temp_root("demo-generated");
    let artifact = write_latest_artifact(&root, agent_latest_artifact(&root));
    let report = execute_mvp_demo_contract(&demo_config(&root, &artifact))
        .expect("demo should include harness proof");

    assert!(report.contains(r#""agent_harness_evidence":""#));
    assert!(report.contains(r#""id":"mcp_agent_harness_verification""#));
    assert!(report.contains(r#""harness":"mcp-agent""#));
    assert!(!report.contains(r#""id":"devnet_settlement_asset_movement""#));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn spec_c07_markdown_report_surfaces_agent_harness_evidence() {
    let root = temp_root("markdown-generated");
    let artifact = write_latest_artifact(
        &root,
        agent_latest_artifact_with_surface(&root, "pi-extension-tools"),
    );
    execute_mvp_demo_contract(&demo_config(&root, &artifact))
        .expect("demo should write markdown report");

    let markdown = std::fs::read_to_string(root.join("latest/proof/report.md"))
        .expect("markdown report should exist");
    assert!(markdown.contains("Agent harness evidence"));
    assert!(markdown.contains("mcp_agent_harness_verification"));
    assert!(markdown.contains("pi-extension-tools"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn spec_c08_project_local_pi_extension_registers_kamn_tools() {
    let source = pi_extension_source();

    for marker in [
        "kamn_verify_mvp_report",
        "agentHarnessEvidencePath",
        "--agent-harness-evidence",
        "kamn_inspect_mvp_report_boundaries",
        "kamn_write_agent_harness_evidence",
        "kamn_run_demo_mvp_with_agent_evidence",
        "three_agent_boundary",
        "three_agent_actor_rehearsal",
        "three_agent_actor_tool_receipts",
        "three_agent_actor_observation_receipts",
        "kamn_agent_a_register",
        "kamn_agent_a_invoke_transaction",
        "kamn_agent_b_register",
        "kamn_agent_b_accept_task",
        "kamn_agent_c_verify_three_agent_proof",
        "invoke_transaction",
        "accept_task",
        "kamn_live_agent_a_register",
        "kamn_live_agent_a_query_profile",
        "query_agent_profile",
        "session_shutdown",
        "KAMN_MVP_LIVE_MCP_BINARY",
        "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE",
        "kamn_live_agent_b_register",
        "kamn_live_agent_a_create_task",
        "kamn_live_agent_b_accept_task",
        "kamn_live_agent_a_query_task",
        "kamn_live_agent_b_query_task",
        "KAMN_MVP_LIVE_MCP_AGENT_B_NAME",
        "KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE",
        "real local-only task lifecycle",
        "kamn_live_agent_a_publish_task_handoff",
        "kamn_live_agent_b_receive_task_handoff",
        "kamn_live_agent_a_wait_for_task_acceptance",
        "kamn_live_agent_b_write_task_receipt",
        "kamn_live_verify_independent_actor_receipts",
        "KAMN_MVP_LIVE_TASK_HANDOFF_FILE",
        "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE",
        "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE",
        "real local-only independent Pi actors",
    ] {
        assert!(source.contains(marker), "missing Pi tool marker: {marker}");
    }
}

#[test]
fn spec_c22_cli_verifies_direct_pi_evidence_without_mutating_report() {
    let root = temp_root("direct-pi-evidence");
    let report = write_report(&root, direct_report_with_three_agent_claim(&root));
    let artifact = write_artifact(
        &root,
        agent_artifact_with_canonical_observation_receipts(&root),
    );
    let before = std::fs::read(&report).expect("report should be readable");

    let output = verify_with_direct_evidence(report.as_path(), artifact.as_path());

    assert!(output.status.success(), "{}", command_output(&output));
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"status\":\"PASS\""));
    assert_eq!(
        before,
        std::fs::read(&report).expect("report should remain readable")
    );
}

#[test]
fn spec_c23_cli_rejects_direct_pi_evidence_for_a_different_report() {
    let root = temp_root("direct-pi-evidence-report-mismatch");
    let report = write_report(&root, direct_report_with_three_agent_claim(&root));
    let artifact_json = agent_artifact_with_canonical_observation_receipts(&root)
        .replace("proof/report.json", "proof/other-report.json");
    let artifact = write_artifact(&root, artifact_json);

    let output = verify_with_direct_evidence(report.as_path(), artifact.as_path());

    assert!(
        !output.status.success(),
        "mismatched report evidence must fail"
    );
    assert!(command_output(&output).contains("report_path does not match"));
}

fn verify_with_direct_evidence(report: &Path, artifact: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_kamn-e2e-harness"))
        .args([
            "verify-mvp-demo",
            "--report",
            report.to_str().expect("report path should be UTF-8"),
            "--agent-harness-evidence",
            artifact.to_str().expect("artifact path should be UTF-8"),
        ])
        .output()
        .expect("verifier binary should execute")
}

fn command_output(output: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn pi_extension_source() -> String {
    [
        ".pi/extensions/kamn-mvp/index.ts",
        ".pi/extensions/kamn-mvp/evidence.ts",
        ".pi/extensions/kamn-mvp/actor-receipts.ts",
        ".pi/extensions/kamn-mvp/live-mcp-tools.ts",
        ".pi/extensions/kamn-mvp/mcp-session.ts",
        ".pi/extensions/kamn-mvp/live-task-workflow.ts",
        ".pi/extensions/kamn-mvp/live-task-coordination.ts",
    ]
    .map(|path| {
        std::fs::read_to_string(workspace_root().join(path))
            .unwrap_or_else(|_| panic!("KAMN Pi extension file should exist: {path}"))
    })
    .join("\n")
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates/")
        .parent()
        .expect("workspace root should contain crates/")
}
