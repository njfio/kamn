use super::{base_agent_name, connect_agent, validate_non_empty, DEFAULT_S05_FUND_ESCROW_PAYLOAD};

pub(super) fn run_live_s05_escrow_settlement_probe() -> Result<(), String> {
    let fund_payload = super::super::env_var_or_default(
        "KAMN_E2E_S05_FUND_ESCROW_PAYLOAD",
        DEFAULT_S05_FUND_ESCROW_PAYLOAD,
    );
    let agent_name = base_agent_name();
    let funded = fund_escrow(&format!("{agent_name}-s05-fund"), fund_payload.as_str())?;
    release_escrow(&format!("{agent_name}-s05-release"), funded.as_str())
}

fn fund_escrow(agent_name: &str, payload: &str) -> Result<String, String> {
    let handle = connect_agent(agent_name, "sdk-direct live s05 connect failed")?;
    let receipt = handle
        .fund_escrow(payload)
        .map_err(|error| format!("sdk-direct live s05 fund-escrow failed: {error}"))?;
    validate_non_empty(
        receipt.escrow_id.as_str(),
        "sdk-direct live s05 fund-escrow returned empty escrow_id",
    )?;
    validate_non_empty(
        receipt.state.as_str(),
        "sdk-direct live s05 fund-escrow returned empty state",
    )?;
    Ok(receipt.escrow_id)
}

fn release_escrow(agent_name: &str, escrow_id: &str) -> Result<(), String> {
    let handle = connect_agent(agent_name, "sdk-direct live s05 connect failed")?;
    let receipt = handle
        .release_escrow(escrow_id)
        .map_err(|error| format!("sdk-direct live s05 release-escrow failed: {error}"))?;
    validate_live_s05_release_escrow_receipt(
        escrow_id,
        receipt.escrow_id.as_str(),
        receipt.state.as_str(),
    )
}

pub(super) fn validate_live_s05_release_escrow_receipt(
    expected_escrow_id: &str,
    released_escrow_id: &str,
    released_state: &str,
) -> Result<(), String> {
    if released_escrow_id != expected_escrow_id {
        return Err(format!(
            "sdk-direct live s05 release-escrow returned mismatched escrow_id: expected={expected_escrow_id}, got={released_escrow_id}"
        ));
    }
    validate_non_empty(
        released_state,
        "sdk-direct live s05 release-escrow returned empty state",
    )
}
