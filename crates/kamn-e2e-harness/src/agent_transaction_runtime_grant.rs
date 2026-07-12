use std::path::{Path, PathBuf};

use kamn_sdk::{service_public_key_for_private_key, AgentDid};

use super::AgentTransactionDemoConfig;

const RUNTIME_ERROR: &str = "AGENT_TRANSACTION_RUNTIME_FAILED";
const AGENT_A_NAME: &str = "kamn-mvp-agent-a";

pub(super) fn write_task_creation_grant(
    config: &AgentTransactionDemoConfig,
) -> Result<PathBuf, String> {
    let signing_key = std::fs::read_to_string(config.agent_key_files[0].as_str())
        .map_err(|error| runtime_error("Agent A key read", error))?;
    let public_key = service_public_key_for_private_key(signing_key.trim())
        .map_err(|error| format!("{RUNTIME_ERROR}: Agent A key invalid: {error}"))?;
    let did = AgentDid::with_public_key_hex_binding(AGENT_A_NAME, public_key.as_str())
        .map_err(|error| format!("{RUNTIME_ERROR}: Agent A DID invalid: {error}"))?;
    let path = Path::new(config.staging_root.as_str()).join("service-api-state.json");
    std::fs::write(&path, state_json(did.as_str()))
        .map_err(|error| runtime_error("grant bootstrap write", error))?;
    Ok(path)
}

fn state_json(did: &str) -> String {
    format!(
        r#"{{"schema_version":"kamn.runtime.service-api-message-store.v4","messages":{{}},"channel_messages":{{}},"agent_grants":{{"agent-a-task-create":{{"did":"{did}","resource":"transaction:new","role":"initiator","action":"task:create","status":"active","idempotency_key":"agent-a-task-create"}}}}}}"#
    )
}

fn runtime_error(context: &str, error: std::io::Error) -> String {
    format!("{RUNTIME_ERROR}: {context} failed: {error}")
}
