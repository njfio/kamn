use std::env;

use super::{
    default_endpoint, kolme_endpoint, KolmeProofReceipt, DEFAULT_S14_AGENT_NAME,
    DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A, DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B,
    DEFAULT_S14_BLOCK_HEIGHT, DEFAULT_S14_FINALITY,
};
use crate::drivers::sdk_direct::live_probe_tranche_three::live_probe_support::{
    query_message_status_with_handle, send_message_with_validated_receipt,
};

pub(super) fn run_live_s14_batch_merkle_probe() -> Result<(), String> {
    let settings = s14_settings()?;
    let batch_ids = send_batch_messages(&settings)?;
    query_batch_messages(&settings, &batch_ids)?;
    let proof_receipt = build_proof_receipt(&settings, &batch_ids)?;
    verify_batch_messages(&settings, &batch_ids, &proof_receipt)
}

struct S14Settings {
    endpoint: String,
    kolme_endpoint: String,
    base_agent_name: String,
    payload_a: String,
    payload_b: String,
    block_height: u64,
    finality: String,
}

struct S14BatchIds {
    batch_a_message_id: String,
    batch_b_message_id: String,
}

fn s14_settings() -> Result<S14Settings, String> {
    Ok(S14Settings {
        endpoint: default_endpoint(),
        kolme_endpoint: kolme_endpoint(),
        base_agent_name: super::super::env_var_or_default(
            "KAMN_E2E_S14_AGENT_NAME",
            DEFAULT_S14_AGENT_NAME,
        ),
        payload_a: super::super::env_var_or_default(
            "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A",
            DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A,
        ),
        payload_b: super::super::env_var_or_default(
            "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B",
            DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B,
        ),
        block_height: parse_s14_block_height()?,
        finality: super::super::env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY),
    })
}

fn parse_s14_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("sdk-direct live s14 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S14_BLOCK_HEIGHT))
}

fn send_batch_messages(settings: &S14Settings) -> Result<S14BatchIds, String> {
    let batch_a_message_id = send_named_batch_message(
        settings,
        "batch-a",
        settings.payload_a.as_str(),
        "sdk-direct live s14 batch-a connect failed",
        "sdk-direct live s14 batch-a send-message",
    )?;
    let batch_b_message_id = send_named_batch_message(
        settings,
        "batch-b",
        settings.payload_b.as_str(),
        "sdk-direct live s14 batch-b connect failed",
        "sdk-direct live s14 batch-b send-message",
    )?;
    super::super::validate_s08_distinct_message_ids(
        batch_a_message_id.as_str(),
        batch_b_message_id.as_str(),
        "sdk-direct live s14 batch-b send-message",
    )?;
    Ok(S14BatchIds {
        batch_a_message_id,
        batch_b_message_id,
    })
}

fn send_named_batch_message(
    settings: &S14Settings,
    suffix: &str,
    payload: &str,
    connect_error: &str,
    step: &str,
) -> Result<String, String> {
    send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        payload,
        connect_error,
        step,
    )
}

fn query_batch_messages(settings: &S14Settings, batch_ids: &S14BatchIds) -> Result<(), String> {
    let query_handle = super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-query", settings.base_agent_name).as_str(),
        "sdk-direct live s14 query connect failed",
    )?;
    query_message_status_with_handle(
        &query_handle,
        batch_ids.batch_a_message_id.as_str(),
        "sdk-direct live s14 batch-a query-message",
    )?;
    query_message_status_with_handle(
        &query_handle,
        batch_ids.batch_b_message_id.as_str(),
        "sdk-direct live s14 batch-b query-message",
    )
}

fn build_proof_receipt(
    settings: &S14Settings,
    batch_ids: &S14BatchIds,
) -> Result<KolmeProofReceipt, String> {
    let batch_root = super::super::env_var_or_else("KAMN_E2E_S14_BATCH_ROOT", || {
        format!(
            "sha256:s14:{}:{}",
            batch_ids.batch_a_message_id, batch_ids.batch_b_message_id
        )
    });
    super::validate_non_empty(
        batch_root.as_str(),
        "sdk-direct live s14 batch-root marker must not be empty",
    )?;
    Ok(KolmeProofReceipt {
        tx_hash: batch_root,
        block_height: settings.block_height,
        finality: settings.finality.clone(),
    })
}

fn verify_batch_messages(
    settings: &S14Settings,
    batch_ids: &S14BatchIds,
    proof_receipt: &KolmeProofReceipt,
) -> Result<(), String> {
    let proof_handle = super::connect_agent(
        settings.endpoint.as_str(),
        settings.kolme_endpoint.as_str(),
        format!("{}-proof", settings.base_agent_name).as_str(),
        "sdk-direct live s14 proof connect failed",
    )?;
    verify_batch_proof(
        &proof_handle,
        batch_ids.batch_a_message_id.as_str(),
        proof_receipt,
        "sdk-direct live s14 batch-a verify-proof",
    )?;
    verify_batch_proof(
        &proof_handle,
        batch_ids.batch_b_message_id.as_str(),
        proof_receipt,
        "sdk-direct live s14 batch-b verify-proof",
    )
}

fn verify_batch_proof(
    handle: &super::KamnAgentHandle,
    message_id: &str,
    proof_receipt: &KolmeProofReceipt,
    step: &str,
) -> Result<(), String> {
    let verification = handle
        .verify_proof(message_id, proof_receipt)
        .map_err(|error| format!("{step} failed: {error}"))?;
    super::validate_s14_proof_response(
        message_id,
        verification.message_id.as_str(),
        verification.block_height,
        verification.finality.as_str(),
        verification.verified,
        step,
    )
}
