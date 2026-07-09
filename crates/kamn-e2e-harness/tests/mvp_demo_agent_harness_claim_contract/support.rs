use std::path::{Path, PathBuf};

use kamn_e2e_harness::{MvpDemoCommandConfig, VerifyMvpDemoCommandConfig};

mod actor_receipts;
mod artifact;
#[path = "../support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;
mod three_agent;

pub(crate) use artifact::*;
pub(crate) use three_agent::view_digest_for;

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
    mvp_local_artifacts::write_valid_local_artifacts(root);
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
    let transcript = root.join("proof/three-agent-transcript.json");
    write_file(transcript.as_path(), three_agent::valid_transcript(root));
    for (path, content) in three_agent::valid_view_artifacts(root) {
        write_file(path.as_path(), content);
    }
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"run-7047","status":"GO","devnet_mode":"required","artifacts":{},"claim_matrix":[{},{},{},{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json_with_three_agent(root, Some(artifact), transcript.as_path()),
        local_claims(),
        three_agent::devnet_settlement_claim(),
        three_agent::three_agent_claim(root, transcript.as_path()),
        agent_claim(),
        roadmap_claim()
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
{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass","signed_exchange":{"verified": true}}
JSON
echo "receipt_reconciliation=GO"
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
        format!(r#"echo "test {test_name} ... ok"; echo "test result: ok""#),
    ]
}

fn artifacts_json(root: &Path, agent_artifact: Option<&Path>) -> String {
    let mut artifacts = mvp_local_artifacts::artifacts_json(root, None);
    if let Some(path) = agent_artifact {
        artifacts.pop();
        artifacts.push_str(format!(r#","agent_harness_evidence":"{}"}}"#, path.display()).as_str());
    }
    artifacts
}

fn artifacts_json_with_three_agent(
    root: &Path,
    agent_artifact: Option<&Path>,
    transcript: &Path,
) -> String {
    let mut artifacts = artifacts_json(root, agent_artifact);
    artifacts.pop();
    artifacts.push_str(
        format!(
            r#","three_agent_transcript":"{}","agent_a_view":"{}","agent_b_view":"{}","agent_c_verifier_view":"{}"}}"#,
            transcript.display(),
            root.join("proof/agent-a-view.json").display(),
            root.join("proof/agent-b-view.json").display(),
            root.join("proof/agent-c-verifier-view.json").display()
        )
        .as_str(),
    );
    artifacts
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
