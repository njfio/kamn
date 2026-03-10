use std::env;

use super::{
    connect_agent, default_agent_name, default_endpoint, kolme_endpoint, KolmeProofReceipt,
    DEFAULT_S06_BLOCK_HEIGHT, DEFAULT_S06_FINALITY, DEFAULT_S06_MESSAGE_ID, DEFAULT_S06_TX_HASH,
    DEFAULT_S07_AGENT_NAME, DEFAULT_S07_MESSAGE_PAYLOAD,
};

pub(super) fn run_live_s06_proof_verification_probe() -> Result<(), String> {
    let endpoint = default_endpoint();
    let kolme_endpoint = kolme_endpoint();
    let agent_name = default_agent_name();
    let message_id =
        super::super::env_var_or_default("KAMN_E2E_S06_PROOF_MESSAGE_ID", DEFAULT_S06_MESSAGE_ID);
    let receipt = s06_receipt()?;
    let handle = connect_agent(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
        "sdk-direct live s06 connect failed",
    )?;
    let verification = handle
        .verify_proof(message_id.as_str(), &receipt)
        .map_err(|error| format!("sdk-direct live s06 verify-proof failed: {error}"))?;
    require_s06_verification(verification.verified, verification.finality.as_str())
}

pub(super) fn run_live_s07_replay_protection_probe() -> Result<(), String> {
    let endpoint = default_endpoint();
    let kolme_endpoint = kolme_endpoint();
    let agent_name = replay_agent_name();
    let payload = super::super::env_var_or_default(
        "KAMN_E2E_S07_REPLAY_PAYLOAD",
        DEFAULT_S07_MESSAGE_PAYLOAD,
    );
    send_s07_initial_message(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
        payload.as_str(),
    )?;
    reject_s07_replay(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
        payload.as_str(),
    )
}

fn s06_receipt() -> Result<KolmeProofReceipt, String> {
    Ok(KolmeProofReceipt {
        tx_hash: super::super::env_var_or_default(
            "KAMN_E2E_S06_PROOF_TX_HASH",
            DEFAULT_S06_TX_HASH,
        ),
        block_height: s06_block_height()?,
        finality: super::super::env_var_or_default(
            "KAMN_E2E_S06_PROOF_FINALITY",
            DEFAULT_S06_FINALITY,
        ),
    })
}

fn s06_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s06 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S06_BLOCK_HEIGHT))
}

fn require_s06_verification(verified: bool, finality: &str) -> Result<(), String> {
    if !verified {
        return Err("sdk-direct live s06 verify-proof returned verified=false".to_owned());
    }
    if finality.trim() != "FINAL" {
        return Err(format!(
            "sdk-direct live s06 verify-proof returned non-final finality: {finality}"
        ));
    }
    Ok(())
}

fn replay_agent_name() -> String {
    let base_agent_name =
        super::super::env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    format!(
        "{base_agent_name}-{}",
        super::super::live_s07_probe_agent_suffix().as_str()
    )
}

fn send_s07_initial_message(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    payload: &str,
) -> Result<(), String> {
    let handle = connect_agent(
        endpoint,
        kolme_endpoint,
        agent_name,
        "sdk-direct live s07 connect failed",
    )?;
    let receipt = handle
        .send_message(payload)
        .map_err(|error| format!("sdk-direct live s07 initial send-message failed: {error}"))?;
    super::validate_non_empty(
        receipt.message_id.as_str(),
        "sdk-direct live s07 initial send-message returned empty message_id",
    )?;
    super::validate_non_empty(
        receipt.status.as_str(),
        "sdk-direct live s07 initial send-message returned empty status",
    )
}

fn reject_s07_replay(
    endpoint: &str,
    kolme_endpoint: &str,
    agent_name: &str,
    payload: &str,
) -> Result<(), String> {
    let handle = connect_agent(
        endpoint,
        kolme_endpoint,
        agent_name,
        "sdk-direct live s07 connect failed",
    )?;
    let replay_error = handle.send_message(payload).err().ok_or_else(|| {
        "sdk-direct live s07 replay send-message unexpectedly succeeded".to_owned()
    })?;
    super::super::validate_s07_replay_reason_marker(
        replay_error.to_string().as_str(),
        "sdk-direct live s07 replay send-message",
    )
}
