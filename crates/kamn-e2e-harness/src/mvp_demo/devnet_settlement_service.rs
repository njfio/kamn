use std::path::Path;

use super::live_task_binding::LiveTaskBinding;
use kamn_agent_lib::KamnAgentHandle;

const SDK_TIMEOUT_ENV: &str = "KAMN_SDK_SERVICE_TIMEOUT_SECONDS";
const LIVE_SETTLEMENT_TIMEOUT_SECONDS: &str = "90";

pub(super) fn drive_escrow_release(
    endpoint: &str,
    run_dir: &Path,
    binding: Option<&LiveTaskBinding>,
) -> Result<String, String> {
    let _timeout = EnvOverride::set(SDK_TIMEOUT_ENV, LIVE_SETTLEMENT_TIMEOUT_SECONDS);
    let handle = KamnAgentHandle::connect(
        endpoint,
        "http://127.0.0.1:13000",
        "kamn-mvp-devnet-settlement",
    )
    .map_err(|error| format!("failed to create KAMN agent handle: {error}"))?;
    let payload = fund_payload(run_dir, binding)?;
    write_funding_payload(run_dir, payload.as_str())?;
    let funded = handle
        .fund_escrow(payload.as_str())
        .map_err(|error| format!("failed to fund MVP demo escrow: {error}"))?;
    require_expected_escrow_id(payload.as_str(), funded.escrow_id.as_str())?;
    let released = handle
        .release_escrow(funded.escrow_id.as_str())
        .map_err(|error| format!("failed to release MVP demo escrow: {error}"))?;
    require_released(released.state.as_str())?;
    Ok(released.escrow_id)
}

fn require_released(state: &str) -> Result<(), String> {
    if state == "released" {
        return Ok(());
    }
    Err(format!(
        "devnet settlement escrow state not released: {state}"
    ))
}

struct EnvOverride {
    name: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvOverride {
    fn set(name: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(name);
        std::env::set_var(name, value);
        Self { name, previous }
    }
}

impl Drop for EnvOverride {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => std::env::set_var(self.name, value),
            None => std::env::remove_var(self.name),
        }
    }
}

fn fund_payload(run_dir: &Path, binding: Option<&LiveTaskBinding>) -> Result<String, String> {
    let run_id = run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "failed to derive MVP demo run id for escrow payload".to_owned())?;
    match binding {
        Some(value) => Ok(format!(
            "{{\"schema_version\":\"kamn.mvp.devnet-settlement.v2\",\"run_id\":\"{run_id}\",\"task_id\":\"{}\",\"task_binding_digest\":\"{}\"}}",
            value.task_id, value.digest
        )),
        None => Ok(format!(
            "{{\"schema_version\":\"kamn.mvp.devnet-settlement.v1\",\"run_id\":\"{run_id}\"}}"
        )),
    }
}

fn write_funding_payload(run_dir: &Path, payload: &str) -> Result<(), String> {
    std::fs::write(
        run_dir.join("proof/devnet-escrow-funding-request.json"),
        payload,
    )
    .map_err(|error| format!("failed to write devnet escrow funding request: {error}"))
}

fn require_expected_escrow_id(payload: &str, escrow_id: &str) -> Result<(), String> {
    let expected = expected_escrow_id(payload);
    if escrow_id == expected {
        Ok(())
    } else {
        Err(format!(
            "devnet settlement escrow ID mismatch: expected {expected}, found {escrow_id}"
        ))
    }
}

pub(super) fn expected_escrow_id(payload: &str) -> String {
    format!(
        "escrow-local-{:016x}",
        deterministic_body_tag(payload.as_bytes())
    )
}

fn deterministic_body_tag(payload: &[u8]) -> u64 {
    payload.iter().fold(0xcbf29ce484222325_u64, |acc, byte| {
        acc.wrapping_mul(0x00000100000001B3) ^ u64::from(*byte)
    })
}
