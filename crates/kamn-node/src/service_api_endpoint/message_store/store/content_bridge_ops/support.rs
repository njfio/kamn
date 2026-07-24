use super::super::super::*;

pub(super) fn bridge_source_message_id_from_payload(
    payload: &str,
    bridge_tag: u64,
    bridge_id: &str,
) -> String {
    let default_value = format!("msg-bridge-source-{bridge_tag:016x}");
    let parsed = match serde_json::from_str::<serde_json::Value>(payload) {
        Ok(value) => value,
        Err(_) => return default_value,
    };
    let Some(source_message_id) = parsed
        .get("source_message_id")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return default_value;
    };
    if source_message_id == bridge_id {
        return default_value;
    }
    source_message_id.to_owned()
}

pub(super) fn next_content_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_content_bridge_id("content-local", payload, |candidate| {
        store.snapshot.contents.contains_key(candidate)
    })
}

pub(super) fn next_bridge_id(store: &ServiceApiMessageStore, bridge_tag: u64) -> String {
    let base = format!("bridge-local-{bridge_tag:016x}");
    let mut candidate = base.clone();
    let mut suffix = 1_u64;
    while store.snapshot.bridges.contains_key(candidate.as_str()) {
        candidate = format!("{base}-{suffix}");
        suffix = suffix.saturating_add(1);
    }
    candidate
}

fn next_local_content_bridge_id<F>(prefix: &str, payload: &str, exists: F) -> String
where
    F: Fn(&str) -> bool,
{
    let base = format!(
        "{prefix}-{:016x}",
        deterministic_body_tag(payload.as_bytes())
    );
    let mut candidate = base.clone();
    let mut suffix = 1_u64;
    while exists(candidate.as_str()) {
        candidate = format!("{base}-{suffix}");
        suffix = suffix.saturating_add(1);
    }
    candidate
}

pub(super) fn build_content_record(content_id: &str) -> ServiceApiPersistedContentRecord {
    ServiceApiPersistedContentRecord {
        content_id: content_id.to_owned(),
        retention_class: "standard".to_owned(),
        lifecycle_state: "retained".to_owned(),
        redaction_status: "none".to_owned(),
    }
}

pub(super) fn content_register_body(content_id: String) -> ServiceApiContentRegisterBody {
    ServiceApiContentRegisterBody {
        content_id,
        retention_class: "standard".to_owned(),
        lifecycle_state: "retained".to_owned(),
        redaction_status: "none".to_owned(),
    }
}

pub(super) fn build_bridge_record(
    bridge_id: &str,
    source_message_id: &str,
    payload: &str,
) -> Result<ServiceApiPersistedBridgeRecord, String> {
    Ok(ServiceApiPersistedBridgeRecord {
        bridge_id: bridge_id.to_owned(),
        source_message_id: source_message_id.to_owned(),
        bridge_status: "submitted".to_owned(),
        target_message_id: format!("msg-bridge-target-{bridge_id}"),
        forward_tx_hash: String::new(),
        target_network: "solana:devnet".to_owned(),
        payload_hash: super::super::super::authority_digest::bridge_payload(payload),
        settlement_authority: settlement_authority_from_payload(payload)?,
        prepared_transaction: None,
        bridge_receipt: None,
        submission_attempt_count: 0,
        last_error_code: None,
    })
}

fn settlement_authority_from_payload(
    payload: &str,
) -> Result<Option<ServiceApiBridgeSettlementTermsRecord>, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(payload) else {
        return Ok(None);
    };
    let Some(authority) = value.get("settlement_authority") else {
        return Ok(None);
    };
    serde_json::from_value(authority.clone())
        .map(Some)
        .map_err(|error| format!("BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH: {error}"))
}

pub(super) fn bridge_submit_body(
    bridge_id: String,
    source_message_id: String,
) -> ServiceApiBridgeSubmitBody {
    ServiceApiBridgeSubmitBody {
        bridge_id,
        source_message_id,
        bridge_status: "submitted".to_owned(),
    }
}
