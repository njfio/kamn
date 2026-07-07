use std::path::Path;

use kamn_agent_lib::KamnAgentHandle;

const SDK_TIMEOUT_ENV: &str = "KAMN_SDK_SERVICE_TIMEOUT_SECONDS";
const LIVE_SETTLEMENT_TIMEOUT_SECONDS: &str = "90";

pub(super) fn drive_escrow_release(endpoint: &str, run_dir: &Path) -> Result<String, String> {
    let _timeout = EnvOverride::set(SDK_TIMEOUT_ENV, LIVE_SETTLEMENT_TIMEOUT_SECONDS);
    let handle = KamnAgentHandle::connect(
        endpoint,
        "http://127.0.0.1:13000",
        "kamn-mvp-devnet-settlement",
    )
    .map_err(|error| format!("failed to create KAMN agent handle: {error}"))?;
    let payload = fund_payload(run_dir)?;
    let funded = handle
        .fund_escrow(payload.as_str())
        .map_err(|error| format!("failed to fund MVP demo escrow: {error}"))?;
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

fn fund_payload(run_dir: &Path) -> Result<String, String> {
    let run_id = run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "failed to derive MVP demo run id for escrow payload".to_owned())?;
    Ok(format!(
        "{{\"schema_version\":\"kamn.mvp.devnet-settlement.v1\",\"run_id\":\"{run_id}\"}}"
    ))
}
