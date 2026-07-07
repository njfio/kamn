use std::path::Path;
use std::time::{Duration, Instant};

use super::devnet_settlement_json::json_string_value;

pub(super) fn persisted_signature(state_file: &Path, escrow_id: &str) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match try_persisted_signature(state_file, escrow_id) {
            Ok(signature) => return Ok(signature),
            Err(error) => last_error = error,
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(last_error)
}

fn try_persisted_signature(state_file: &Path, escrow_id: &str) -> Result<String, String> {
    let state = std::fs::read_to_string(state_file)
        .map_err(|error| format!("failed to read service API settlement state: {error}"))?;
    let escrow_state = escrow_state_slice(state.as_str(), escrow_id)?;
    require_persisted_settlement_fields(escrow_state)?;
    json_string_value(escrow_state, "settlement_tx_signature")
}

fn escrow_state_slice<'a>(state: &'a str, escrow_id: &str) -> Result<&'a str, String> {
    let marker = format!("\"{escrow_id}\":");
    let start = state
        .find(marker.as_str())
        .ok_or_else(|| format!("persisted escrow not found in state: {escrow_id}"))?;
    Ok(&state[start..])
}

fn require_persisted_settlement_fields(state: &str) -> Result<(), String> {
    let network = json_string_value(state, "settlement_network")?;
    if network != "solana:devnet" {
        return Err(format!(
            "unexpected persisted settlement network: {network}"
        ));
    }
    json_string_value(state, "settlement_commitment")?;
    Ok(())
}
