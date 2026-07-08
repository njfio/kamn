use std::path::{Path, PathBuf};

use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};

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
fn spec_c03_command_rejects_verifier_private_leak_in_agent_harness_artifact() {
    let root = temp_root("private-leak");
    let artifact = write_artifact(&root, agent_artifact(&root, true, "devnet-backed"));
    let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("agent harness verifier view must stay restricted");

    assert!(err.contains("verifier_private_view_visible"));
}

#[test]
fn spec_c04_command_rejects_local_only_settlement_in_agent_harness_artifact() {
    let root = temp_root("local-settlement");
    let artifact = write_artifact(&root, agent_artifact(&root, false, "local-only"));
    let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("agent harness cannot count local-only settlement success");

    assert!(err.contains("settlement_claim_label"));
}

#[test]
fn spec_c05_command_accepts_valid_agent_harness_artifact() {
    let root = temp_root("valid");
    let artifact = write_artifact(&root, agent_artifact(&root, false, "devnet-backed"));
    let report = write_report(&root, report_with_agent_claim(&root, Some(&artifact)));

    execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("valid MCP-agent harness evidence should verify");
}

fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7047-{stem}-{}-{millis}", std::process::id()))
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
    }
}

fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    std::fs::create_dir_all(path.parent().expect("report parent should exist")).unwrap();
    std::fs::write(&path, report).unwrap();
    path
}

fn write_artifact(root: &Path, artifact: String) -> PathBuf {
    let path = root.join("proof/agent-harness-evidence.json");
    std::fs::create_dir_all(path.parent().expect("artifact parent should exist")).unwrap();
    std::fs::write(&path, artifact).unwrap();
    path
}

fn direct_report(root: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, None),
        local_claims(),
        roadmap_claim()
    )
}

fn report_with_agent_claim(root: &Path, artifact: Option<&Path>) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, artifact),
        local_claims(),
        agent_claim(),
        roadmap_claim()
    )
}

fn artifacts_json(root: &Path, agent_artifact: Option<&Path>) -> String {
    let base = root.join("proof");
    let mut fields = format!(
        r#""report_json":"{}","report_md":"{}","state_dir":"{}","audit_export":"{}","localhost_signed_demo_artifact":"{}","localhost_signed_demo_output":"{}","service_api_vertical_slice_output":"{}","service_api_websocket_output":"{}","devnet_settlement_output":"{}""#,
        base.join("report.json").display(),
        base.join("report.md").display(),
        root.join("state").display(),
        base.join("audit-export.json").display(),
        base.join("localhost-signed-demo.json").display(),
        base.join("localhost-signed-demo-output.txt").display(),
        base.join("service-api-vertical-slice-output.txt").display(),
        base.join("service-api-websocket-output.txt").display(),
        base.join("devnet-settlement-output.txt").display()
    );
    if let Some(path) = agent_artifact {
        fields.push_str(format!(r#","agent_harness_evidence":"{}""#, path.display()).as_str());
    }
    format!("{{{fields}}}")
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local runtime"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"agent identities"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"message flow"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket events"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit export"}"#
}

fn agent_claim() -> &'static str {
    r#"{"id":"mcp_agent_harness_verification","label":"local-only","required":false,"status":"PASS","summary":"MCP agent harness verified report boundaries","harness":"mcp-agent"}"#
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}

fn agent_artifact(root: &Path, private_visible: bool, settlement_label: &str) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.agent-harness-evidence.v1","harness":"mcp-agent","execution_surface":"mcp-tools","report_path":"{}","verifier_status":"PASS","participant_agents":["agent_a","agent_b","agent_c_verifier"],"tool_markers":["register","create_task","fund_escrow","release_escrow","verify_proof"],"claim_boundaries":{{"settlement_claim_label":"{}","dry_run_counted_as_success":false,"placeholder_counted_as_success":false,"verifier_private_view_visible":{}}}}}"#,
        root.join("proof/report.json").display(),
        settlement_label,
        private_visible
    )
}
