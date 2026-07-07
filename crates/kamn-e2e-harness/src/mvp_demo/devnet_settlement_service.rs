use std::path::Path;

use kamn_agent_lib::KamnAgentHandle;

pub(super) fn drive_escrow_release(endpoint: &str, run_dir: &Path) -> Result<String, String> {
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

fn fund_payload(run_dir: &Path) -> Result<String, String> {
    let run_id = run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "failed to derive MVP demo run id for escrow payload".to_owned())?;
    Ok(format!(
        "{{\"schema_version\":\"kamn.mvp.devnet-settlement.v1\",\"run_id\":\"{run_id}\"}}"
    ))
}
