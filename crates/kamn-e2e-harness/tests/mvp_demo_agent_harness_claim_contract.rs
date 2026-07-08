#[path = "mvp_demo_agent_harness_claim_contract/support.rs"]
mod support;

use kamn_e2e_harness::{execute_mvp_demo_contract, execute_verify_mvp_demo_contract};
use std::path::Path;
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
    let extension = workspace_root().join(".pi/extensions/kamn-mvp/index.ts");
    let source = std::fs::read_to_string(extension).expect("KAMN Pi extension should exist");

    for marker in [
        "kamn_verify_mvp_report",
        "kamn_inspect_mvp_report_boundaries",
        "kamn_write_agent_harness_evidence",
        "kamn_run_demo_mvp_with_agent_evidence",
    ] {
        assert!(source.contains(marker), "missing Pi tool marker: {marker}");
    }
}

#[test]
fn spec_c09_command_rejects_agent_artifact_without_three_agent_boundary() {
    let root = temp_root("missing-three-agent-boundary");
    let artifact = write_artifact(&root, agent_artifact_without_three_agent_boundary(&root));
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("agent harness evidence must include three-agent boundary");

    assert!(err.contains("three_agent_boundary"));
}

#[test]
fn spec_c10_command_rejects_agent_artifact_with_mismatched_three_agent_claim() {
    for (stem, from, to, expected) in [
        ("status", r#""claim_status":"PASS""#, r#""claim_status":"NO-GO""#, "claim_status"),
        (
            "label",
            r#""claim_label":"devnet-backed""#,
            r#""claim_label":"local-only""#,
            "claim_label",
        ),
    ] {
        let root = temp_root(format!("mismatched-three-agent-{stem}").as_str());
        let artifact_json = agent_artifact_with_three_agent_boundary(&root).replace(from, to);
        let artifact = write_artifact(&root, artifact_json);
        let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

        let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
            .expect_err("agent harness evidence must match report claim");
        assert!(err.contains(expected));
    }
}

#[test]
fn spec_c11_command_rejects_agent_artifact_with_invalid_private_boundary() {
    for (stem, from, to, expected) in [
        (
            "agent-a-zero",
            r#""agent_a_private_field_count":3"#,
            r#""agent_a_private_field_count":0"#,
            "agent_a_private_field_count",
        ),
        (
            "verifier-private",
            r#""verifier_private_field_count":0"#,
            r#""verifier_private_field_count":1"#,
            "verifier_private_field_count",
        ),
        (
            "digest-present",
            r#""verifier_private_view_digest_present":false"#,
            r#""verifier_private_view_digest_present":true"#,
            "verifier_private_view_digest_present",
        ),
        (
            "unredacted",
            r#""private_payload_redacted":true"#,
            r#""private_payload_redacted":false"#,
            "private_payload_redacted",
        ),
    ] {
        let root = temp_root(format!("invalid-three-agent-{stem}").as_str());
        let artifact_json = agent_artifact_with_three_agent_boundary(&root).replace(from, to);
        let artifact = write_artifact(&root, artifact_json);
        let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

        let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
            .expect_err("agent harness evidence must preserve private boundary");
        assert!(err.contains(expected));
    }
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates/")
        .parent()
        .expect("workspace root should contain crates/")
}
