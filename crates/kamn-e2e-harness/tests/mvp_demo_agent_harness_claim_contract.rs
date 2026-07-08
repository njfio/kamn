#[path = "mvp_demo_agent_harness_claim_contract/support.rs"]
mod support;

use kamn_e2e_harness::{execute_mvp_demo_contract, execute_verify_mvp_demo_contract};
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
fn spec_c05_demo_mvp_can_include_agent_harness_evidence() {
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
