#![allow(dead_code)]

const DEFAULT_RESPONSE: &str = r#"{"channel_id":"unknown","messages":[]}"#;

fn health_and_channel_routes(method: &str, path: &str) -> Option<(u16, &'static str)> {
    match (method, path) {
        ("GET", "/healthz") => Some((
            200,
            r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
        )),
        ("POST", "/v1/messages/send") => Some((
            202,
            r#"{"message_id":"msg-cli","status":"created","runtime_mode":"api"}"#,
        )),
        ("POST", "/v1/channels/create") => {
            Some((201, r#"{"channel_id":"channel-cli","status":"created"}"#))
        }
        ("GET", "/v1/channels/channel-cli/messages") => Some((
            200,
            r#"{"channel_id":"channel-cli","messages":["msg-1","msg-2"]}"#,
        )),
        _ => None,
    }
}

fn task_and_profile_routes(method: &str, path: &str) -> Option<(u16, &'static str)> {
    match (method, path) {
        ("GET", "/v1/messages/msg-cli") => {
            Some((200, r#"{"message_id":"msg-cli","status":"created"}"#))
        }
        ("GET", "/v1/tasks/task-cli") => {
            Some((200, r#"{"task_id":"task-cli","state":"submitted"}"#))
        }
        ("GET", "/v1/agents/kamn:did:agent:alice") => Some((
            200,
            r#"{"did":"kamn:did:agent:alice","reputation_score":777,"agent_type":"service-agent","model_family":"service-api","capabilities":["profile:read"],"profile_commitment":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#,
        )),
        ("POST", "/v1/tasks/create") => Some((
            201,
            r#"{"task_id":"task-cli","state":"submitted","receipt_id":"task-receipt-1","receipt_digest":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","action":"task:create"}"#,
        )),
        ("POST", "/v1/tasks/task-cli/accept") => Some((
            200,
            r#"{"task_id":"task-cli","state":"accepted","receipt_id":"task-receipt-2","receipt_digest":"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","action":"task:accept"}"#,
        )),
        ("POST", "/v1/tasks/task-cli/complete") => Some((
            200,
            r#"{"task_id":"task-cli","state":"completed","receipt_id":"task-receipt-3","receipt_digest":"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","action":"task:complete"}"#,
        )),
        ("POST", "/v1/escrow/fund") => Some((
            200,
            r#"{"escrow_id":"escrow-cli","state":"funded","receipt_id":"escrow-receipt-1","receipt_digest":"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd","action":"escrow:fund"}"#,
        )),
        ("POST", "/v1/escrow/escrow-cli/release") => Some((
            200,
            r#"{"escrow_id":"escrow-cli","state":"released","receipt_id":"escrow-receipt-2","receipt_digest":"sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","action":"escrow:release-authorize"}"#,
        )),
        _ => None,
    }
}

fn content_routes(method: &str, path: &str) -> Option<(u16, &'static str)> {
    match (method, path) {
        ("POST", "/v1/content/register") => Some((
            201,
            r#"{"content_id":"content-cli","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#,
        )),
        ("POST", "/v1/content/content-cli/expire") => Some((
            200,
            r#"{"content_id":"content-cli","lifecycle_state":"expired","redaction_status":"none"}"#,
        )),
        ("POST", "/v1/content/content-cli/tombstone") | ("GET", "/v1/content/content-cli") => {
            Some((
                200,
                r#"{"content_id":"content-cli","lifecycle_state":"tombstoned","redaction_status":"redacted"}"#,
            ))
        }
        _ => None,
    }
}

fn bridge_routes(method: &str, path: &str) -> Option<(u16, &'static str)> {
    match (method, path) {
        ("POST", "/v1/bridge/submit") => Some((
            202,
            r#"{"bridge_id":"bridge-cli","source_message_id":"msg-bridge-source-cli","bridge_status":"submitted"}"#,
        )),
        ("POST", "/v1/bridge/bridge-cli/forward") | ("GET", "/v1/bridge/bridge-cli") => Some((
            200,
            r#"{"bridge_id":"bridge-cli","bridge_status":"forwarded","target_message_id":"msg-bridge-target-cli","forward_tx_hash":"sha256:bridge-forwarded-cli"}"#,
        )),
        _ => None,
    }
}

pub(crate) fn response_for(method: &str, path: &str) -> (u16, &'static str) {
    health_and_channel_routes(method, path)
        .or_else(|| task_and_profile_routes(method, path))
        .or_else(|| content_routes(method, path))
        .or_else(|| bridge_routes(method, path))
        .unwrap_or((200, DEFAULT_RESPONSE))
}
