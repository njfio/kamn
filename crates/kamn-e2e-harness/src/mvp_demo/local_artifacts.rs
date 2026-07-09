use std::path::{Path, PathBuf};

pub(crate) fn create_demo_artifacts(run_dir: &Path) -> Result<(), String> {
    create_dir(run_dir.join("state").as_path())?;
    create_dir(run_dir.join("events").as_path())?;
    create_dir(run_dir.join("proof").as_path())?;
    write_file(
        run_dir.join("state/runtime-state.json"),
        runtime_state_json(),
    )?;
    write_file(run_dir.join("state/relay-projection.json"), relay_json())?;
    write_file(run_dir.join("events/websocket-events.json"), events_json())?;
    write_file(run_dir.join("proof/audit-export.json"), audit_json())
}

fn create_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|error| {
        format!(
            "failed to create MVP demo directory {}: {error}",
            path.display()
        )
    })
}

fn write_file(path: PathBuf, content: &str) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| {
        format!(
            "failed to write MVP demo artifact {}: {error}",
            path.display()
        )
    })
}

fn runtime_state_json() -> &'static str {
    r#"{"runtime":"kamn-local","source":"localhost-signed-demo","alice":"kamn:did:agent:alice","bob":"kamn:did:agent:bob","signed_flow":"task"}"#
}

fn relay_json() -> &'static str {
    r#"{"relay_state":"projected","source":"service-api-vertical-slice","message_status":"delivered","durable_state":"written"}"#
}

fn events_json() -> &'static str {
    r#"{"source":"service-api-websocket","events":["service-api.message.created","service-api.task.completed"]}"#
}

fn audit_json() -> &'static str {
    r#"{"audit_export":"service-api-runtime-export","source":"service-api-vertical-slice","records":["service_api_task_created"]}"#
}
