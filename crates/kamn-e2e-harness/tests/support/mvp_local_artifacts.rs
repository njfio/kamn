use std::path::Path;

pub fn write_valid_local_artifacts(root: &Path) {
    write_file(
        root.join("proof/localhost-signed-demo.json").as_path(),
        r#"{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass","signed_exchange":{"from":"kamn:did:agent:alice","to":"kamn:did:agent:bob","verified": true},"signed_flow":"task"}"#,
    );
    write_file(
        root.join("proof/localhost-signed-demo-output.txt")
            .as_path(),
        "receipt_reconciliation=GO\nlocalhost signed message demo completed.\n",
    );
    write_file(
        root.join("proof/service-api-vertical-slice-output.txt").as_path(),
        "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence ... ok\ntest result: ok\n",
    );
    write_file(
        root.join("proof/service-api-websocket-output.txt").as_path(),
        "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event ... ok\ntest result: ok\n",
    );
    write_file(
        root.join("proof/audit-export.json").as_path(),
        r#"{"audit_export":"service-api-runtime-export","source":"service-api-vertical-slice","records":["service_api_task_created"]}"#,
    );
    write_file(
        root.join("state/runtime-state.json").as_path(),
        r#"{"runtime":"kamn-local","source":"localhost-signed-demo","alice":"kamn:did:agent:alice","bob":"kamn:did:agent:bob","signed_flow":"task"}"#,
    );
    write_file(
        root.join("state/relay-projection.json").as_path(),
        r#"{"relay_state":"projected","source":"service-api-vertical-slice","message_status":"delivered","durable_state":"written"}"#,
    );
    write_file(
        root.join("events/websocket-events.json").as_path(),
        r#"{"source":"service-api-websocket","events":["service-api.message.created","service-api.task.completed"]}"#,
    );
    write_file(
        root.join("proof/devnet-settlement-output.txt").as_path(),
        "devnet_settlement_status=SKIP reason=devnet_mode_optional\n",
    );
}

pub fn artifacts_json(root: &Path, transcript: Option<&Path>) -> String {
    let base = format!(
        r#"{{"report_json":"{}","report_md":"{}","state_dir":"{}","audit_export":"{}","localhost_signed_demo_artifact":"{}","localhost_signed_demo_output":"{}","service_api_vertical_slice_output":"{}","service_api_websocket_output":"{}","devnet_settlement_output":"{}"}}"#,
        root.join("proof/report.json").display(),
        root.join("proof/report.md").display(),
        root.join("state").display(),
        root.join("proof/audit-export.json").display(),
        root.join("proof/localhost-signed-demo.json").display(),
        root.join("proof/localhost-signed-demo-output.txt")
            .display(),
        root.join("proof/service-api-vertical-slice-output.txt")
            .display(),
        root.join("proof/service-api-websocket-output.txt")
            .display(),
        root.join("proof/devnet-settlement-output.txt").display()
    );
    match transcript {
        Some(path) => format!(
            "{},\"three_agent_transcript\":\"{}\"}}",
            base.trim_end_matches('}'),
            path.display()
        ),
        None => base,
    }
}

pub fn write_file(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().expect("parent should exist"))
        .expect("fixture directory should be created");
    std::fs::write(path, content).expect("fixture should be written");
}
