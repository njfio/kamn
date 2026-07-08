use std::path::{Path, PathBuf};

use kamn_e2e_harness::{MvpDemoCommandConfig, VerifyMvpDemoCommandConfig};

mod three_agent;

pub(crate) fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7047-{stem}-{}-{millis}", std::process::id()))
}

pub(crate) fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
    }
}

pub(crate) fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    write_file(path.as_path(), report);
    path
}

pub(crate) fn write_artifact(root: &Path, artifact: String) -> PathBuf {
    let path = root.join("proof/agent-harness-evidence.json");
    write_file(path.as_path(), artifact);
    path
}

pub(crate) fn write_latest_artifact(root: &Path, artifact: String) -> PathBuf {
    let path = root.join("agent-harness-evidence.json");
    write_file(path.as_path(), artifact);
    path
}

pub(crate) fn demo_config(root: &Path, artifact: &Path) -> MvpDemoCommandConfig {
    MvpDemoCommandConfig {
        output_root: root.display().to_string(),
        devnet_mode: "optional".to_owned(),
        solana_rpc_url: None,
        devnet_settlement_command: None,
        localhost_signed_demo_command: Some(stub_localhost_command()),
        service_api_vertical_slice_command: Some(stub_service_command("integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence")),
        service_api_websocket_command: Some(stub_service_command("integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event")),
        agent_harness_evidence_path: Some(artifact.display().to_string()),
    }
}

pub(crate) fn direct_report(root: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, None),
        local_claims(),
        roadmap_claim()
    )
}

pub(crate) fn report_with_agent_claim(root: &Path, artifact: Option<&Path>) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, artifact),
        local_claims(),
        agent_claim(),
        roadmap_claim()
    )
}

pub(crate) fn report_with_three_agent_claim(root: &Path, artifact: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{},{},{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root, Some(artifact)),
        local_claims(),
        three_agent::devnet_settlement_claim(),
        three_agent::three_agent_claim(),
        agent_claim(),
        roadmap_claim()
    )
}

pub(crate) fn agent_artifact(root: &Path, private_visible: bool, settlement_label: &str) -> String {
    agent_artifact_with_surface(root, private_visible, settlement_label, "mcp-tools")
}

pub(crate) fn agent_artifact_without_three_agent_boundary(root: &Path) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        "pi-extension-tools",
        three_agent::NO_THREE_AGENT_BOUNDARY,
    )
}

pub(crate) fn agent_artifact_with_surface(
    root: &Path,
    private_visible: bool,
    settlement_label: &str,
    execution_surface: &str,
) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        private_visible,
        settlement_label,
        execution_surface,
        three_agent::absent_boundary(),
    )
}

pub(crate) fn agent_latest_artifact(root: &Path) -> String {
    agent_latest_artifact_with_surface(root, "mcp-tools")
}

pub(crate) fn agent_latest_artifact_with_surface(root: &Path, execution_surface: &str) -> String {
    agent_artifact_for_report_with_surface(
        root.join("latest/proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        execution_surface,
        three_agent::absent_boundary(),
    )
}

pub(crate) fn agent_artifact_with_three_agent_boundary(root: &Path) -> String {
    agent_artifact_for_report_with_surface(
        root.join("proof/report.json")
            .display()
            .to_string()
            .as_str(),
        false,
        "devnet-backed",
        "pi-extension-tools",
        three_agent::valid_boundary(),
    )
}

fn write_file(path: &Path, content: String) {
    std::fs::create_dir_all(path.parent().expect("parent should exist"))
        .expect("fixture parent directory should be creatable");
    std::fs::write(path, content).expect("fixture file should be writable");
}

fn stub_localhost_command() -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        r#"cat > "$2" <<'JSON'
{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass"}
JSON
echo "localhost signed message demo completed."
"#
        .to_owned(),
        "kamn-mvp-stub".to_owned(),
    ]
}

fn stub_service_command(test_name: &str) -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        format!(r#"echo "test {test_name} ... ok""#),
    ]
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

fn agent_artifact_for_report_with_surface(
    report_path: &str,
    private_visible: bool,
    settlement_label: &str,
    execution_surface: &str,
    three_agent_boundary: &str,
) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.agent-harness-evidence.v1","harness":"mcp-agent","execution_surface":"{}","report_path":"{}","verifier_status":"PASS","participant_agents":["agent_a","agent_b","agent_c_verifier"],"tool_markers":["register","create_task","fund_escrow","release_escrow","verify_proof"],"claim_boundaries":{{"settlement_claim_label":"{}","dry_run_counted_as_success":false,"placeholder_counted_as_success":false,"verifier_private_view_visible":{}}}{}}}"#,
        execution_surface,
        report_path,
        settlement_label,
        private_visible,
        three_agent_boundary
    )
}
