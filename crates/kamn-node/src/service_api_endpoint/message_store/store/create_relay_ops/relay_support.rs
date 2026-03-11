use super::super::super::*;
use super::recipient_mailbox_channel_id;

pub(super) struct RelayRequest<'a> {
    pub(super) message_id: &'a str,
    pub(super) sender_did: Option<&'a str>,
    pub(super) recipient_did: &'a str,
}

pub(super) fn validate_relay_request<'a>(
    message_id: &'a str,
    sender_did: Option<&'a str>,
    recipient_did: &'a str,
) -> Result<RelayRequest<'a>, String> {
    let message_id = message_id.trim();
    if message_id.is_empty() {
        return Err("relay message id must not be empty".to_owned());
    }
    let recipient_did = recipient_did.trim();
    if recipient_did.is_empty() {
        return Err("relay recipient did must not be empty".to_owned());
    }
    Ok(RelayRequest {
        message_id,
        sender_did: sender_did.map(str::trim).filter(|value| !value.is_empty()),
        recipient_did,
    })
}

pub(super) fn upsert_relay_message_record(
    store: &mut ServiceApiMessageStore,
    request: &RelayRequest<'_>,
    body: &str,
) -> Result<bool, String> {
    match store.snapshot.messages.get_mut(request.message_id) {
        Some(record) => update_existing_relay_record(record, request, body),
        None => {
            insert_relay_record(store, request, body);
            Ok(true)
        }
    }
}

pub(super) fn ensure_relay_mailbox_membership(
    store: &mut ServiceApiMessageStore,
    request: &RelayRequest<'_>,
) -> bool {
    let mailbox = store
        .snapshot
        .channel_messages
        .entry(recipient_mailbox_channel_id(request.recipient_did))
        .or_default();
    if mailbox
        .iter()
        .any(|candidate| candidate == request.message_id)
    {
        return false;
    }
    mailbox.push(request.message_id.to_owned());
    true
}

pub(super) fn relay_message_body(
    store: &ServiceApiMessageStore,
    message_id: &str,
) -> ServiceApiMessageRelayBody {
    let status = store
        .snapshot
        .messages
        .get(message_id)
        .map(|record| record.status.clone())
        .unwrap_or_else(|| "relayed".to_owned());
    ServiceApiMessageRelayBody {
        message_id: message_id.to_owned(),
        status,
    }
}

fn update_existing_relay_record(
    record: &mut ServiceApiPersistedMessageRecord,
    request: &RelayRequest<'_>,
    body: &str,
) -> Result<bool, String> {
    validate_relay_recipient(record, request)?;
    validate_relay_body(record, request.message_id, body)?;
    let sender_mutated = maybe_update_relay_sender(record, request)?;
    let status_mutated = maybe_mark_relayed(record);
    Ok(sender_mutated || status_mutated)
}

fn validate_relay_recipient(
    record: &ServiceApiPersistedMessageRecord,
    request: &RelayRequest<'_>,
) -> Result<(), String> {
    if record.recipient_did.as_deref() == Some(request.recipient_did) {
        return Ok(());
    }
    Err(format!(
        "relay recipient mismatch for {}: expected={}, actual={}",
        request.message_id,
        record.recipient_did.as_deref().unwrap_or("none"),
        request.recipient_did
    ))
}

fn validate_relay_body(
    record: &ServiceApiPersistedMessageRecord,
    message_id: &str,
    body: &str,
) -> Result<(), String> {
    if record.body.as_deref() == Some(body) {
        return Ok(());
    }
    Err(format!(
        "relay body mismatch for {message_id}: existing payload differs"
    ))
}

fn maybe_update_relay_sender(
    record: &mut ServiceApiPersistedMessageRecord,
    request: &RelayRequest<'_>,
) -> Result<bool, String> {
    let Some(sender) = request.sender_did else {
        return Ok(false);
    };
    match record.sender_did.as_deref() {
        Some(existing) if existing != sender => Err(format!(
            "relay sender mismatch for {}: expected={existing}, actual={sender}",
            request.message_id
        )),
        None => {
            record.sender_did = Some(sender.to_owned());
            Ok(true)
        }
        _ => Ok(false),
    }
}

fn maybe_mark_relayed(record: &mut ServiceApiPersistedMessageRecord) -> bool {
    if record.status.as_str() != "created" {
        return false;
    }
    record.status = "relayed".to_owned();
    true
}

fn insert_relay_record(store: &mut ServiceApiMessageStore, request: &RelayRequest<'_>, body: &str) {
    store.snapshot.messages.insert(
        request.message_id.to_owned(),
        ServiceApiPersistedMessageRecord {
            message_id: request.message_id.to_owned(),
            status: "relayed".to_owned(),
            channel_id: None,
            sender_did: request.sender_did.map(str::to_owned),
            recipient_did: Some(request.recipient_did.to_owned()),
            body: Some(body.to_owned()),
            data_layer_runtime_evidence: None,
        },
    );
}
