use super::super::*;
use kamn_core::CANONICAL_ENCRYPTION_ALGORITHM;

pub(crate) struct SliceEvidence<'a> {
    pub(crate) recipient_did: &'a str,
    pub(crate) message_id: &'a str,
    pub(crate) task_id: &'a str,
    pub(crate) mailbox: &'a crate::service_api_endpoint::ServiceApiChannelMessagesBody,
    pub(crate) delivered_message: &'a Value,
    pub(crate) queried_task: &'a Value,
    pub(crate) sender_state: &'a Value,
    pub(crate) audit_export: &'a Value,
}

pub(crate) fn assert_slice_evidence(evidence: SliceEvidence<'_>) {
    assert_eq!(CANONICAL_ENCRYPTION_ALGORITHM, "X25519-XChaCha20-Poly1305");
    assert!(evidence
        .mailbox
        .messages
        .contains(&evidence.message_id.to_owned()));
    assert_eq!(evidence.delivered_message["status"], "delivered");
    assert_eq!(
        evidence.delivered_message["recipient_did"],
        evidence.recipient_did
    );
    assert_runtime_evidence(&evidence.sender_state["messages"][evidence.message_id]);
    assert_task_completion(
        evidence.task_id,
        evidence.queried_task,
        &evidence.sender_state["tasks"][evidence.task_id],
    );
    assert_task_audit_record(evidence.task_id, evidence.audit_export);
}

pub(crate) fn recipient_env_guards(
    state_file: &std::path::Path,
    spool_file: &std::path::Path,
) -> ((String, EnvVarGuard), (String, EnvVarGuard)) {
    (
        set_state_file_env(state_file),
        set_relay_spool_env(spool_file),
    )
}

fn assert_runtime_evidence(sender_message: &Value) {
    let runtime_evidence = &sender_message["data_layer_runtime_evidence"];
    assert_eq!(
        runtime_evidence["schema_version"],
        "kamn.runtime.service-api-data-layer-runtime-evidence.v1",
    );
    assert_hash_prefix(runtime_evidence, "m0_content_hash");
    assert_hash_prefix(runtime_evidence, "m1_merkle_root");
}

fn assert_task_completion(task_id: &str, queried_task: &Value, persisted_task: &Value) {
    assert_eq!(queried_task["state"], "completed");
    assert_eq!(persisted_task["state"], "completed");
    assert_eq!(persisted_task["task_id"], task_id);
}

fn assert_task_audit_record(task_id: &str, audit_export: &Value) {
    let task_record = audit_export["records"]
        .as_array()
        .and_then(|records| {
            records
                .iter()
                .find(|record| is_task_create_record(record, task_id))
        })
        .expect("audit export should contain the task-create record");
    assert_eq!(task_record["action"], "service_api_task_created");
    assert_eq!(task_record["event_id"], task_id);
}

fn assert_hash_prefix(runtime_evidence: &Value, field: &str) {
    assert!(runtime_evidence[field]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
}

fn is_task_create_record(record: &Value, task_id: &str) -> bool {
    record["action"] == "service_api_task_created" && record["event_id"] == task_id
}
