use super::super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

fn render_snapshot() -> ServiceApiSnapshot {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34051".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    build_service_api_snapshot(&report)
}

fn assert_response(snapshot: &ServiceApiSnapshot, method: &str, path: &str, body: &str, status: u16) {
    let response = render_service_api_endpoint_response(snapshot, method, path, body);
    assert_eq!(response.status_code, status, "{method} {path} status");
}

#[test]
fn functional_service_api_endpoint_renders_required_route_contracts() {
    let snapshot = render_snapshot();
    assert_message_and_task_routes(&snapshot);
    assert_content_and_bridge_routes(&snapshot);
    let health_response = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    assert_eq!(health_response.status_code, 200);
    assert_agent_and_health_routes(&snapshot);
}

fn assert_message_and_task_routes(snapshot: &ServiceApiSnapshot) {
    let send_response = render_service_api_endpoint_response(
        snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert_eq!(send_response.status_code, 202);
    assert!(send_response.body.contains("\"message_id\":\"msg-local-"));
    let read_response = render_service_api_endpoint_response(snapshot, "GET", "/v1/messages/msg-7", "");
    assert_eq!(read_response.status_code, 200);
    assert!(read_response.body.contains("\"status\":\"created\""));
    assert_response(snapshot, "GET", "/v1/channels/channel-1/messages", "", 200);
    assert_response(snapshot, "GET", "/v1/tasks/task-1", "", 200);
    let accepted = render_service_api_endpoint_response(snapshot, "POST", "/v1/tasks/task-1/accept", "{}");
    assert_eq!(accepted.status_code, 200);
    assert!(accepted.body.contains("\"state\":\"accepted\""));
    let completed = render_service_api_endpoint_response(snapshot, "POST", "/v1/tasks/task-1/complete", "{}");
    assert_eq!(completed.status_code, 200);
    assert!(completed.body.contains("\"state\":\"completed\""));
}

fn assert_content_and_bridge_routes(snapshot: &ServiceApiSnapshot) {
    let escrow_fund_response = render_service_api_endpoint_response(snapshot, "POST", "/v1/escrow/fund", "{\"task_id\":\"task-1\",\"amount\":100}");
    assert_eq!(escrow_fund_response.status_code, 200);
    assert!(escrow_fund_response.body.contains("\"escrow_id\":\"escrow-local-"));
    assert!(escrow_fund_response.body.contains("\"state\":\"funded\""));
    let escrow_release_response = render_service_api_endpoint_response(snapshot, "POST", "/v1/escrow/escrow-1/release", "{}");
    assert_eq!(escrow_release_response.status_code, 200);
    assert!(escrow_release_response.body.contains("\"state\":\"released\""));
    let content_register_response = render_service_api_endpoint_response(snapshot, "POST", "/v1/content/register", "{\"content\":\"hello\"}");
    assert_eq!(content_register_response.status_code, 201);
    assert!(content_register_response.body.contains("\"content_id\":\"content-local-"));
    assert!(content_register_response.body.contains("\"retention_class\":\"standard\""));
    assert_route_lifecycle_queries(snapshot);
}

fn assert_route_lifecycle_queries(snapshot: &ServiceApiSnapshot) {
    let expired = render_service_api_endpoint_response(snapshot, "POST", "/v1/content/content-1/expire", "{}");
    assert!(expired.body.contains("\"lifecycle_state\":\"expired\""));
    let tombstoned = render_service_api_endpoint_response(snapshot, "POST", "/v1/content/content-1/tombstone", "{}");
    assert!(tombstoned.body.contains("\"redaction_status\":\"redacted\""));
    let content = render_service_api_endpoint_response(snapshot, "GET", "/v1/content/content-1", "");
    assert!(content.body.contains("\"lifecycle_state\":\"tombstoned\""));
    let bridge_submit_response = render_service_api_endpoint_response(snapshot, "POST", "/v1/bridge/submit", "{\"source_message_id\":\"msg-1\",\"target_network\":\"testnet\"}");
    assert_eq!(bridge_submit_response.status_code, 202);
    assert!(bridge_submit_response.body.contains("\"bridge_id\":\"bridge-local-"));
    assert!(bridge_submit_response.body.contains("\"bridge_status\":\"submitted\""));
    let bridge_forward_response = render_service_api_endpoint_response(snapshot, "POST", "/v1/bridge/bridge-1/forward", "{}");
    assert!(bridge_forward_response.body.contains("\"bridge_status\":\"forwarded\""));
    assert!(bridge_forward_response.body.contains("\"target_message_id\":\"msg-bridge-target-bridge-1\""));
    let bridge_query_response = render_service_api_endpoint_response(snapshot, "GET", "/v1/bridge/bridge-1", "");
    assert!(bridge_query_response.body.contains("\"forward_tx_hash\":\"sha256:bridge-forwarded-bridge-1\""));
}

fn assert_agent_and_health_routes(snapshot: &ServiceApiSnapshot) {
    let agent_response = render_service_api_endpoint_response(
        snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    assert_eq!(agent_response.status_code, 200);
}
