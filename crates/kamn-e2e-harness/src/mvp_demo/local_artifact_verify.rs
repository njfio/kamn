use std::path::{Path, PathBuf};

use super::local_artifact_paths::LocalArtifactPaths;
use super::verify_support::require_marker;

const LOCALHOST_SCHEMA: &str = "kamn.sdk.localhost-signed.demo-receipt-artifact.v1";
const LOCALHOST_SUCCESS: &str = "localhost signed message demo completed.";
const VERTICAL_SLICE_TEST: &str =
    "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence";
const WEBSOCKET_TEST: &str =
    "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event";

pub(crate) fn validate_local_artifact_files(report_json: &str) -> Result<(), String> {
    let paths = LocalArtifactPaths::from_report(report_json)?;
    validate_localhost_artifact(paths.localhost_artifact.as_path())?;
    validate_localhost_output(paths.localhost_output.as_path())?;
    validate_service_log(
        paths.vertical_log.as_path(),
        VERTICAL_SLICE_TEST,
        "service API vertical slice",
    )?;
    validate_service_log(
        paths.websocket_log.as_path(),
        WEBSOCKET_TEST,
        "service API websocket",
    )?;
    validate_state_dir(paths.state_dir.as_path())?;
    validate_audit_export(paths.audit_export.as_path())?;
    validate_devnet_log(paths.devnet_log.as_path())
}

fn validate_localhost_artifact(path: &Path) -> Result<(), String> {
    let content = read_file(path, "localhost signed demo artifact")?;
    require_marker(
        content.as_str(),
        LOCALHOST_SCHEMA,
        "localhost signed demo artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"status\": \"pass\"",
        "localhost signed demo artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"signed_exchange\"",
        "localhost signed demo artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"verified\": true",
        "localhost signed demo artifact",
    )
}

fn validate_localhost_output(path: &Path) -> Result<(), String> {
    let content = read_file(path, "localhost signed demo output")?;
    require_marker(
        content.as_str(),
        LOCALHOST_SUCCESS,
        "localhost signed demo output",
    )?;
    require_marker(
        content.as_str(),
        "receipt_reconciliation=GO",
        "localhost signed demo output",
    )
}

fn validate_service_log(path: &Path, test_name: &str, label: &str) -> Result<(), String> {
    let content = read_file(path, label)?;
    require_marker(content.as_str(), test_name, label)?;
    require_marker(content.as_str(), "test result: ok", label)
}

fn validate_state_dir(path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!(
            "missing local state_dir artifact: {}",
            path.display()
        ));
    }
    validate_runtime_state(path.join("runtime-state.json").as_path())?;
    validate_relay_projection(path.join("relay-projection.json").as_path())?;
    validate_websocket_events(events_path(path).as_path())
}

fn validate_runtime_state(path: &Path) -> Result<(), String> {
    let content = read_file(path, "runtime state artifact")?;
    require_marker(
        content.as_str(),
        "\"runtime\":\"kamn-local\"",
        "runtime state artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"source\":\"localhost-signed-demo\"",
        "runtime state artifact",
    )?;
    require_marker(
        content.as_str(),
        "kamn:did:agent:alice",
        "runtime state artifact",
    )?;
    require_marker(
        content.as_str(),
        "kamn:did:agent:bob",
        "runtime state artifact",
    )
}

fn validate_relay_projection(path: &Path) -> Result<(), String> {
    let content = read_file(path, "relay projection artifact")?;
    require_marker(
        content.as_str(),
        "\"relay_state\":\"projected\"",
        "relay projection artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"source\":\"service-api-vertical-slice\"",
        "relay projection artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"message_status\":\"delivered\"",
        "relay projection artifact",
    )
}

fn validate_websocket_events(path: &Path) -> Result<(), String> {
    let content = read_file(path, "websocket events artifact")?;
    require_marker(
        content.as_str(),
        "\"source\":\"service-api-websocket\"",
        "websocket events artifact",
    )?;
    require_marker(
        content.as_str(),
        "service-api.message.created",
        "websocket events artifact",
    )?;
    require_marker(
        content.as_str(),
        "service-api.task.completed",
        "websocket events artifact",
    )
}

fn validate_audit_export(path: &Path) -> Result<(), String> {
    let content = read_file(path, "audit export artifact")?;
    require_marker(
        content.as_str(),
        "\"audit_export\":\"service-api-runtime-export\"",
        "audit export artifact",
    )?;
    require_marker(
        content.as_str(),
        "\"source\":\"service-api-vertical-slice\"",
        "audit export artifact",
    )?;
    require_marker(
        content.as_str(),
        "service_api_task_created",
        "audit export artifact",
    )
}

fn validate_devnet_log(path: &Path) -> Result<(), String> {
    let content = read_file(path, "devnet settlement output")?;
    if content.contains("devnet_settlement_status=SKIP")
        || content.contains("devnet_settlement_status=PASS")
        || content.contains("devnet_settlement_status=NO-GO")
    {
        return Ok(());
    }
    Err("missing MVP demo report marker for devnet settlement output".to_owned())
}

fn events_path(state_dir: &Path) -> PathBuf {
    state_dir
        .parent()
        .map(|run_dir| run_dir.join("events/websocket-events.json"))
        .unwrap_or_else(|| state_dir.join("../events/websocket-events.json"))
}

fn read_file(path: &Path, label: &str) -> Result<String, String> {
    std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {label} {}: {error}", path.display()))
}
