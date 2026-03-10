use serde_json::json;
use std::path::Path;

pub(super) fn write_json_fixture(
    path: &Path,
    payload: serde_json::Value,
    encode_ctx: &str,
    write_ctx: &str,
) {
    std::fs::write(
        path,
        serde_json::to_string_pretty(&payload).expect(encode_ctx),
    )
    .expect(write_ctx);
}

pub(super) fn message_state_payload(
    message_id: &str,
    status: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
) -> serde_json::Value {
    json!({
        "schema_version": "kamn.runtime.service-api-message-store.v2",
        "messages": {
            message_id: {
                "message_id": message_id,
                "status": status,
                "channel_id": serde_json::Value::Null,
                "sender_did": sender_did,
                "recipient_did": recipient_did,
                "body": body,
            }
        },
        "channel_messages": {},
        "tasks": {},
        "escrows": {},
    })
}

pub(super) fn empty_state_payload() -> serde_json::Value {
    json!({
        "schema_version": "kamn.runtime.service-api-message-store.v2",
        "messages": {},
        "channel_messages": {},
        "tasks": {},
        "escrows": {},
    })
}

pub(super) fn spool_entry_payload(
    message_id: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
    queued_at_unix: u64,
) -> serde_json::Value {
    json!({
        "message_id": message_id,
        "sender_did": sender_did,
        "recipient_did": recipient_did,
        "body": body,
        "queued_at_unix": queued_at_unix,
    })
}
