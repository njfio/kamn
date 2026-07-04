#[path = "vertical_slice_contract_tests/support.rs"]
mod support;

use super::*;
use support::{
    assert_server_ok, assert_slice_evidence, boot_snapshot, create_task, default_audit_export_file,
    list_mailbox_live, project_relay_to_recipient, query_message_live, query_task,
    read_audit_export_json, read_state_json, recipient_env_guards, register_agent_profile,
    send_message, set_audit_export_file_env, set_relay_spool_env, set_state_file_env,
    spawn_api_server, VerticalSliceFiles,
};

#[test]
fn integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence(
) {
    let _env = acquire_service_api_test_env();
    let case = build_case();
    let created_message = send_slice_message(&case);
    let (mailbox, delivered_message) = receive_slice_message(&case, &created_message.message_id);
    let (created_task, queried_task) = dispatch_slice_task(&case);
    // Contract marker: X25519-XChaCha20-Poly1305
    // Contract marker: service_api_task_created
    // Contract marker: completed
    // Contract marker: delivered
    let sender_state = read_state_json(case.files.sender_state_file.as_path());
    let audit_export = read_audit_export_json(case.sender_audit_export.as_path());
    assert_slice_evidence(support::SliceEvidence {
        recipient_did: case.recipient_did.as_str(),
        message_id: &created_message.message_id,
        task_id: &created_task.task_id,
        mailbox: &mailbox,
        delivered_message: &delivered_message,
        queried_task: &queried_task,
        sender_state: &sender_state,
        audit_export: &audit_export,
    });
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

fn send_slice_message(
    case: &VerticalSliceCase,
) -> crate::service_api_endpoint::ServiceApiMessageCreateBody {
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
) -> (
    crate::service_api_endpoint::ServiceApiChannelMessagesBody,
    Value,
) {
    let _recipient_env = recipient_env_guards(
        case.files.recipient_state_file.as_path(),
        case.files.recipient_spool_file.as_path(),
    );
    let recipient_server = start_recipient_server(case);
    project_sender_relay(case);
    let delivered = query_recipient_delivery(case, message_id);
    assert_recipient_server_ok(recipient_server);
    delivered
}

fn start_recipient_server(case: &VerticalSliceCase) -> thread::JoinHandle<Result<(), String>> {
    let server = spawn_api_server(
        &case.recipient_snapshot,
        case.recipient_bind_addr.as_str(),
        3,
    );
    wait_for_endpoint_ready(case.recipient_bind_addr.as_str());
    server
}

fn project_sender_relay(case: &VerticalSliceCase) {
    project_relay_to_recipient(
        case.files.sender_state_file.as_path(),
        case.files.sender_spool_file.as_path(),
        case.recipient_bind_addr.as_str(),
        case.recipient_did.as_str(),
    );
}

fn query_recipient_delivery(
    case: &VerticalSliceCase,
    message_id: &str,
) -> (
    crate::service_api_endpoint::ServiceApiChannelMessagesBody,
    Value,
) {
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
    (mailbox, delivered_message)
}

fn assert_recipient_server_ok(server: thread::JoinHandle<Result<(), String>>) {
    assert_server_ok(
        server,
        "recipient service api endpoint should stop cleanly after working vertical slice relay flow",
    );
}

fn dispatch_slice_task(
    case: &VerticalSliceCase,
) -> (crate::service_api_endpoint::ServiceApiTaskCreateBody, Value) {
    register_vertical_slice_worker(case);
    let created_task = create_vertical_slice_task(case);
    let queried_task = query_vertical_slice_task(case, created_task.task_id.as_str());
    (created_task, queried_task)
}

fn register_vertical_slice_worker(case: &VerticalSliceCase) {
    with_sender_env(case, || {
        register_agent_profile(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            "kamn:did:agent:vertical-slice-recipient",
            704,
            r#"{"agent_type":"worker","model_family":"vertical-slice","capabilities":["vertical-slice"]}"#,
        )
    });
}

fn create_vertical_slice_task(
    case: &VerticalSliceCase,
) -> crate::service_api_endpoint::ServiceApiTaskCreateBody {
    with_sender_env(case, || {
        create_task(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            case.sender_did,
            705,
            r#"{"creator":"kamn:did:agent:vertical-slice-sender","task_type":"vertical-slice","description":"prove one working slice"}"#,
        )
    })
}

fn query_vertical_slice_task(case: &VerticalSliceCase, task_id: &str) -> Value {
    with_sender_env(case, || {
        query_task(
            &case.sender_snapshot,
            case.sender_bind_addr.as_str(),
            case.sender_did,
            706,
            task_id,
        )
    })
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
