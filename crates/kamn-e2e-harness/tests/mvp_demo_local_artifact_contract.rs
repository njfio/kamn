use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::{Path, PathBuf};

#[test]
fn spec_c01_command_rejects_missing_local_artifact() {
    let root = temp_root("missing-artifact");
    let report = write_report(&root, local_only_report(&root));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("missing local artifacts must fail verification");

    assert!(err.contains("localhost signed demo artifact"));
}

#[test]
fn spec_c02_command_rejects_tampered_service_api_log() {
    let root = temp_root("tampered-service-log");
    write_valid_local_artifacts(&root);
    write_file(
        root.join("proof/service-api-vertical-slice-output.txt"),
        "--- stdout ---\nmissing real service api test marker\n",
    );
    let report = write_report(&root, local_only_report(&root));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("tampered service API proof log must fail verification");

    assert!(err.contains("service API vertical slice"));
}

#[test]
fn spec_c03_command_accepts_valid_local_only_artifacts() {
    let root = temp_root("valid-local");
    write_valid_local_artifacts(&root);
    let report = write_report(&root, local_only_report(&root));

    let output = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("valid local-only artifact bundle should verify");

    assert!(output.contains("\"status\":\"PASS\""));
}

fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7058-{stem}-{}-{millis}", std::process::id()))
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
    }
}

fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    write_file(path.as_path(), report.as_str());
    path
}

fn write_valid_local_artifacts(root: &Path) {
    write_file(
        root.join("proof/localhost-signed-demo.json"),
        r#"{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass","participants":["alice","bob"],"signed_flow":"task"}"#,
    );
    write_file(
        root.join("proof/localhost-signed-demo-output.txt"),
        "localhost signed message demo completed.\n",
    );
    write_file(
        root.join("proof/service-api-vertical-slice-output.txt"),
        "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence ... ok\ntest result: ok\n",
    );
    write_file(
        root.join("proof/service-api-websocket-output.txt"),
        "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event ... ok\ntest result: ok\n",
    );
    write_file(
        root.join("proof/audit-export.json"),
        r#"{"audit_export":"service-api-runtime-export","source":"service-api-vertical-slice","records":["service_api_task_created"]}"#,
    );
    write_file(
        root.join("state/runtime-state.json"),
        r#"{"runtime":"kamn-local","source":"localhost-signed-demo","alice":"kamn:did:agent:alice","bob":"kamn:did:agent:bob","signed_flow":"task"}"#,
    );
    write_file(
        root.join("state/relay-projection.json"),
        r#"{"relay_state":"projected","source":"service-api-vertical-slice","message_status":"delivered","durable_state":"written"}"#,
    );
    write_file(
        root.join("events/websocket-events.json"),
        r#"{"source":"service-api-websocket","events":["service-api.message.created","service-api.task.completed"]}"#,
    );
    write_file(
        root.join("proof/devnet-settlement-output.txt"),
        "devnet_settlement_status=SKIP reason=devnet_mode_optional\n",
    );
}

fn write_file(path: impl AsRef<Path>, content: &str) {
    let path = path.as_ref();
    std::fs::create_dir_all(path.parent().expect("parent should exist"))
        .expect("fixture directory should be created");
    std::fs::write(path, content).expect("fixture should be written");
}

fn local_only_report(root: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-local-artifacts","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        artifacts_json(root),
        local_claims(),
        roadmap_claim()
    )
}

fn artifacts_json(root: &Path) -> String {
    format!(
        r#"{{"report_json":"{}","report_md":"{}","state_dir":"{}","audit_export":"{}","localhost_signed_demo_artifact":"{}","localhost_signed_demo_output":"{}","service_api_vertical_slice_output":"{}","service_api_websocket_output":"{}","devnet_settlement_output":"{}"}}"#,
        root.join("proof/report.json").display(),
        root.join("proof/report.md").display(),
        root.join("state").display(),
        root.join("proof/audit-export.json").display(),
        root.join("proof/localhost-signed-demo.json").display(),
        root.join("proof/localhost-signed-demo-output.txt").display(),
        root.join("proof/service-api-vertical-slice-output.txt").display(),
        root.join("proof/service-api-websocket-output.txt").display(),
        root.join("proof/devnet-settlement-output.txt").display()
    )
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local runtime"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"agent identities"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"message flow"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket events"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit export"}"#
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
