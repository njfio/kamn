use super::super::support::*;

pub(super) fn assert_message_channel_routes(client: &ServiceApiClient, sender: &AgentDid) {
    let send_payload = r#"{"message":"hello from sdk"}"#;
    let send_response = client
        .send_message(
            send_payload,
            &auth_with_scope(sender, 1, send_payload, "messages:write"),
        )
        .expect("send message should succeed");
    assert!(send_response.message_id.starts_with("msg-local-"));
    assert_eq!(send_response.status, "created");
    assert_message_status(client, sender, send_response.message_id.as_str());
    assert_channel_and_task_routes(client, sender);
}

fn assert_message_status(client: &ServiceApiClient, sender: &AgentDid, message_id: &str) {
    let message_status = client
        .get_message(message_id, &auth_with_scope(sender, 2, "", "messages:read"))
        .expect("get message should succeed");
    assert_eq!(message_status.message_id, message_id);
    assert_eq!(message_status.status, "created");
}

fn assert_channel_and_task_routes(client: &ServiceApiClient, sender: &AgentDid) {
    let channel_payload = r#"{"name":"ops"}"#;
    let channel_response = client
        .create_channel(
            channel_payload,
            &auth_with_scope(sender, 3, channel_payload, "channels:write"),
        )
        .expect("create channel should succeed");
    assert!(channel_response.channel_id.starts_with("channel-local-"));
    let task_payload = r#"{"task":"triage"}"#;
    let task_response = client
        .create_task(
            task_payload,
            &auth_with_scope(sender, 4, task_payload, "tasks:write"),
        )
        .expect("create task should succeed");
    assert!(task_response.task_id.starts_with("task-local-"));
    assert!(task_response
        .receipt_id
        .starts_with("task-transition-receipt-"));
    assert!(task_response.receipt_digest.starts_with("sha256:"));
    assert_task_status(client, sender, task_response.task_id.as_str());
}

fn assert_task_status(client: &ServiceApiClient, sender: &AgentDid, task_id: &str) {
    let task_status = client
        .get_task(task_id, &auth_with_scope(sender, 5, "", "tasks:read"))
        .expect("get task should succeed");
    assert_eq!(task_status.state, "submitted");
}
