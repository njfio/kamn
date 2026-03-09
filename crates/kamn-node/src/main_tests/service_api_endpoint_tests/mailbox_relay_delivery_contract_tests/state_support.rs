use super::super::*;
use std::path::Path;

pub(super) fn read_spool_lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .expect("relay spool file should remain readable")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

pub(super) fn read_first_spool_entry(path: &Path) -> Value {
    let line = read_spool_lines(path)
        .into_iter()
        .next()
        .expect("relay spool should contain at least one entry");
    serde_json::from_str(line.as_str()).expect("relay spool entry should deserialize")
}

pub(super) fn write_relayed_message_fixture(
    path: &Path,
    message_id: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
) {
    let payload = relayed_message_fixture_value(message_id, sender_did, recipient_did, body);
    let json = serde_json::to_string_pretty(&payload).expect("state fixture should serialize");
    std::fs::write(path, json).expect("state fixture should write");
}

fn relayed_message_fixture_value(
    message_id: &str,
    sender_did: &str,
    recipient_did: &str,
    body: &str,
) -> Value {
    serde_json::json!({
        "schema_version": "kamn.runtime.service-api-message-store.v2",
        "messages": {
            message_id: {
                "message_id": message_id,
                "status": "relayed",
                "channel_id": Value::Null,
                "sender_did": sender_did,
                "recipient_did": recipient_did,
                "body": body,
            }
        },
        "channel_messages": {
            format!("recipient:{recipient_did}"): [message_id]
        },
        "tasks": {},
        "escrows": {},
    })
}
