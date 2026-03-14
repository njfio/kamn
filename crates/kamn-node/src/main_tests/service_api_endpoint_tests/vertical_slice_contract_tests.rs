#[path = "vertical_slice_contract_tests/support.rs"]
mod support;

use super::*;
use kamn_core::CANONICAL_ENCRYPTION_ALGORITHM;
use support::{
    assert_server_ok, boot_snapshot, create_task, default_audit_export_file, list_mailbox_live,
    project_relay_to_recipient, query_message_live, query_task, read_audit_export_json,
    read_state_json, register_agent_profile, send_message, set_audit_export_file_env,
    set_relay_spool_env, set_state_file_env, spawn_api_server, VerticalSliceFiles,
};

#[test]
fn integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence() {
    let _env = acquire_service_api_test_env();
    let case = build_case();
    let created_message = send_slice_message(&case);
    let (mailbox, delivered_message) = receive_slice_message(&case, &created_message.message_id);
    let (created_task, queried_task) = dispatch_slice_task(&case);
    assert_slice_evidence(
        &case,
        &created_message.message_id,
        &created_task.task_id,
        &mailbox,
        &delivered_message,
        &queried_task,
    );
    cleanup_case(case);
}

struct VerticalSliceCase {
    files: VerticalSliceFiles,
    sender_audit_export: std::path::PathBuf,
    sender_snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    sender_bind_addr: String,
    recipient_snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    recipient_bind_addr: String,
    sender_did: &'static str,
    recipient_did: String,
}

fn build_case() -> VerticalSliceCase {
    let files = VerticalSliceFiles::new();
    VerticalSliceCase {
        sender_audit_export: default_audit_export_file(files.sender_state_file.as_path()),
        sender_snapshot: boot_snapshot("127.0.0.1:34161"),
        sender_bind_addr: reserve_loopback_addr(),
        recipient_snapshot: boot_snapshot("127.0.0.1:34162"),
        recipient_bind_addr: reserve_loopback_addr(),
        sender_did: "kamn:did:agent:vertical-slice-sender",
        recipient_did: test_service_api_sender_did("kamn:did:agent:vertical-slice-recipient"),
        files,
    }
}

fn send_slice_message(case: &VerticalSliceCase) -> crate::service_api_endpoint::ServiceApiMessageCreateBody {
    let payload = format!(
        r#"{{"recipient_did":"{}","message":"vertical-slice-message"}}"#,
        case.recipient_did,
    );
    with_sender_env(case, || {
        send_message(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            case.sender_did,
            701,
            payload.as_str(),
        )
    })
}

fn receive_slice_message(
    case: &VerticalSliceCase,
    message_id: &str,
) -> (crate::service_api_endpoint::ServiceApiChannelMessagesBody, Value) {
    let (_recipient_state_text, _recipient_state_guard) =
        set_state_file_env(case.files.recipient_state_file.as_path());
    let (_recipient_spool_text, _recipient_spool_guard) =
        set_relay_spool_env(case.files.recipient_spool_file.as_path());
    let recipient_server = spawn_api_server(&case.recipient_snapshot, case.recipient_bind_addr.as_str(), 3);
    wait_for_endpoint_ready(case.recipient_bind_addr.as_str());
    project_relay_to_recipient(
        case.files.sender_state_file.as_path(),
        case.files.sender_spool_file.as_path(),
        case.recipient_bind_addr.as_str(),
        case.recipient_did.as_str(),
    );
    let mailbox = list_mailbox_live(
        &case.recipient_snapshot,
        case.recipient_bind_addr.as_str(),
        case.recipient_did.as_str(),
        702,
        case.recipient_did.as_str(),
    );
    let delivered_message = query_message_live(
        &case.recipient_snapshot,
        case.recipient_bind_addr.as_str(),
        case.recipient_did.as_str(),
        703,
        message_id,
    );
    assert_server_ok(
        recipient_server,
        "recipient service api endpoint should stop cleanly after working vertical slice relay flow",
    );
    (mailbox, delivered_message)
}

fn dispatch_slice_task(case: &VerticalSliceCase) -> (crate::service_api_endpoint::ServiceApiTaskCreateBody, Value) {
    with_sender_env(case, || {
        register_agent_profile(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            "kamn:did:agent:vertical-slice-recipient",
            704,
            r#"{"agent_type":"worker","model_family":"vertical-slice","capabilities":["vertical-slice"]}"#,
        )
    });
    let created_task = with_sender_env(case, || {
        create_task(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            case.sender_did,
            705,
            r#"{"creator":"kamn:did:agent:vertical-slice-sender","task_type":"vertical-slice","description":"prove one working slice"}"#,
        )
    });
    let queried_task = with_sender_env(case, || {
        query_task(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            case.sender_did,
            706,
            created_task.task_id.as_str(),
        )
    });
    (created_task, queried_task)
}

fn assert_slice_evidence(
    case: &VerticalSliceCase,
    message_id: &str,
    task_id: &str,
    mailbox: &crate::service_api_endpoint::ServiceApiChannelMessagesBody,
    delivered_message: &Value,
    queried_task: &Value,
) {
    let sender_state = read_state_json(case.files.sender_state_file.as_path());
    let sender_message = &sender_state["messages"][message_id];
    let persisted_task = &sender_state["tasks"][task_id];
    let audit_export = read_audit_export_json(case.sender_audit_export.as_path());

    assert_eq!(CANONICAL_ENCRYPTION_ALGORITHM, "X25519-XChaCha20-Poly1305");
    assert!(mailbox.messages.contains(&message_id.to_owned()));
    assert_eq!(delivered_message["status"], "delivered");
    assert_eq!(delivered_message["recipient_did"], case.recipient_did);
    assert_eq!(sender_message["data_layer_runtime_evidence"]["schema_version"], "kamn.runtime.service-api-data-layer-runtime-evidence.v1");
    assert!(sender_message["data_layer_runtime_evidence"]["m0_content_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(sender_message["data_layer_runtime_evidence"]["m1_merkle_root"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert_eq!(queried_task["state"], "completed");
    assert_eq!(persisted_task["state"], "completed");
    let task_record = audit_export["records"]
        .as_array()
        .and_then(|records| {
            records.iter().find(|record| {
                record["action"] == "service_api_task_created" && record["event_id"] == task_id
            })
        })
        .expect("audit export should contain the task-create record");
    assert_eq!(task_record["action"], "service_api_task_created");
    assert_eq!(task_record["event_id"], task_id);
}

fn cleanup_case(case: VerticalSliceCase) {
    case.files.cleanup();
    let _ = fs::remove_file(case.sender_audit_export);
}

fn with_sender_env<T>(case: &VerticalSliceCase, op: impl FnOnce() -> T) -> T {
    let (_sender_state_text, _sender_state_guard) =
        set_state_file_env(case.files.sender_state_file.as_path());
    let (_sender_spool_text, _sender_spool_guard) =
        set_relay_spool_env(case.files.sender_spool_file.as_path());
    let (_sender_audit_text, _sender_audit_guard) =
        set_audit_export_file_env(case.sender_audit_export.as_path());
    op()
}
