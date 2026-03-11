use super::super::super::*;
use super::recipient_mailbox_channel_id;

pub(super) fn next_message_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_id("msg-local", payload, |candidate| {
        store.snapshot.messages.contains_key(candidate)
    })
}

pub(super) fn next_channel_id(store: &ServiceApiMessageStore, payload: &str) -> String {
    next_local_id("channel-local", payload, |candidate| {
        store.snapshot.channel_messages.contains_key(candidate)
    })
}

fn next_local_id<F>(prefix: &str, payload: &str, exists: F) -> String
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

pub(super) fn insert_created_message(
    store: &mut ServiceApiMessageStore,
    message_id: &str,
    payload: &str,
    channel_id: Option<&str>,
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
    data_layer_runtime_evidence: ServiceApiDataLayerRuntimeEvidenceRecord,
) {
    store.snapshot.messages.insert(
        message_id.to_owned(),
        ServiceApiPersistedMessageRecord {
            message_id: message_id.to_owned(),
            status: "created".to_owned(),
            channel_id: channel_id.map(str::to_owned),
            sender_did: sender_did.map(str::to_owned),
            recipient_did: recipient_did.map(str::to_owned),
            body: Some(payload.to_owned()),
            data_layer_runtime_evidence: Some(data_layer_runtime_evidence),
        },
    );
    append_message_channels(store, message_id, channel_id, recipient_did);
}

fn append_message_channels(
    store: &mut ServiceApiMessageStore,
    message_id: &str,
    channel_id: Option<&str>,
    recipient_did: Option<&str>,
) {
    if let Some(channel_id) = channel_id {
        store
            .snapshot
            .channel_messages
            .entry(channel_id.to_owned())
            .or_default()
            .push(message_id.to_owned());
    }
    if let Some(recipient_did) = recipient_did {
        store
            .snapshot
            .channel_messages
            .entry(recipient_mailbox_channel_id(recipient_did))
            .or_default()
            .push(message_id.to_owned());
    }
}

pub(super) fn message_create_body(
    message_id: String,
    runtime_mode: &str,
) -> ServiceApiMessageCreateBody {
    ServiceApiMessageCreateBody {
        message_id,
        status: "created".to_owned(),
        runtime_mode: runtime_mode.to_owned(),
    }
}
