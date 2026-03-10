use std::env;

use super::{
    default_endpoint, env_payload, env_value, DEFAULT_S14_AGENT_NAME,
    DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A, DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B,
    DEFAULT_S14_BLOCK_HEIGHT, DEFAULT_S14_FINALITY,
};
use crate::drivers::cli_scripted::live_probe_tranche_three::live_probe_support::{
    query_message_status, send_message_with_validated_receipt,
};

pub(super) fn run_live_s14_cli_batch_merkle_probe() -> Result<(), String> {
    let settings = s14_settings()?;
    let batch_ids = send_batch_messages(&settings)?;
    query_batch_messages(&settings, &batch_ids)?;
    let proof_args = proof_args(&settings, &batch_ids)?;
    verify_batch_messages(&settings, &batch_ids, &proof_args)
}

struct S14Settings {
    endpoint: String,
    base_agent_name: String,
    payload_a: String,
    payload_b: String,
    block_height: String,
    finality: String,
}

struct S14BatchIds {
    batch_a_message_id: String,
    batch_b_message_id: String,
}

struct S14ProofArgs {
    batch_root: String,
    block_height: String,
    finality: String,
}

fn s14_settings() -> Result<S14Settings, String> {
    Ok(S14Settings {
        endpoint: default_endpoint(),
        base_agent_name: env_value("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME),
        payload_a: env_payload(
            "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A",
            DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A,
        ),
        payload_b: env_payload(
            "KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B",
            DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B,
        ),
        block_height: parse_s14_block_height()?.to_string(),
        finality: env_value("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY),
    })
}

fn parse_s14_block_height() -> Result<u64, String> {
    env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s14 invalid block height env value: {raw}"))
        })
        .transpose()
        .map(|value| value.unwrap_or(DEFAULT_S14_BLOCK_HEIGHT))
}

fn send_batch_messages(settings: &S14Settings) -> Result<S14BatchIds, String> {
    let batch_a_message_id =
        send_named_batch_message(settings, "batch-a", settings.payload_a.as_str())?;
    let batch_b_message_id =
        send_named_batch_message(settings, "batch-b", settings.payload_b.as_str())?;
    super::super::validate_s08_distinct_message_ids(
        batch_a_message_id.as_str(),
        batch_b_message_id.as_str(),
        "cli live s14 batch-b send-message",
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
) -> Result<String, String> {
    send_message_with_validated_receipt(
        settings.endpoint.as_str(),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
        payload,
        &format!("cli live s14 {suffix} send-message"),
    )
}

fn query_batch_messages(settings: &S14Settings, batch_ids: &S14BatchIds) -> Result<(), String> {
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-query-a", settings.base_agent_name).as_str(),
        batch_ids.batch_a_message_id.as_str(),
        "cli live s14 batch-a query-message",
    )?;
    query_message_status(
        settings.endpoint.as_str(),
        format!("{}-query-b", settings.base_agent_name).as_str(),
        batch_ids.batch_b_message_id.as_str(),
        "cli live s14 batch-b query-message",
    )
}

fn proof_args(settings: &S14Settings, batch_ids: &S14BatchIds) -> Result<S14ProofArgs, String> {
    let batch_root = super::super::env_var_or_else("KAMN_E2E_S14_BATCH_ROOT", || {
        format!(
            "sha256:s14:{}:{}",
            batch_ids.batch_a_message_id, batch_ids.batch_b_message_id
        )
    });
    super::validate_non_empty(
        batch_root.as_str(),
        "cli live s14 batch-root marker must not be empty",
    )?;
    Ok(S14ProofArgs {
        batch_root,
        block_height: settings.block_height.clone(),
        finality: settings.finality.clone(),
    })
}

fn verify_batch_messages(
    settings: &S14Settings,
    batch_ids: &S14BatchIds,
    proof_args: &S14ProofArgs,
) -> Result<(), String> {
    verify_batch_message(
        settings,
        batch_ids.batch_a_message_id.as_str(),
        proof_args,
        "proof-a",
    )?;
    verify_batch_message(
        settings,
        batch_ids.batch_b_message_id.as_str(),
        proof_args,
        "proof-b",
    )
}

fn verify_batch_message(
    settings: &S14Settings,
    message_id: &str,
    proof_args: &S14ProofArgs,
    suffix: &str,
) -> Result<(), String> {
    let output = super::super::run_cli_command_capture_stdout_with_agent_name(
        super::cli_binary().as_str(),
        &[
            "verify-proof",
            "--endpoint",
            settings.endpoint.as_str(),
            "--format",
            "text",
            message_id,
            proof_args.batch_root.as_str(),
            proof_args.block_height.as_str(),
            proof_args.finality.as_str(),
        ],
        &format!(
            "cli live s14 batch-{} verify-proof",
            &suffix[suffix.len() - 1..]
        ),
        format!("{}-{suffix}", settings.base_agent_name).as_str(),
    )?;
    super::validate_s14_cli_verify_proof_response(
        output.as_str(),
        message_id,
        &format!(
            "cli live s14 batch-{} verify-proof",
            &suffix[suffix.len() - 1..]
        ),
    )
}
