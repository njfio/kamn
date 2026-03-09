use super::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

#[test]
fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {
    let snapshot = residual_test_snapshot("127.0.0.1:34060");
    assert_health_payload(&snapshot);
    assert_send_payload(&snapshot);
    assert_channel_payload(&snapshot);
    assert_task_payload(&snapshot);
    assert_agent_payload(&snapshot);
}

fn residual_test_snapshot(bind_addr: &str) -> ServiceApiSnapshot {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        bind_addr.to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    build_service_api_snapshot(&report)
}

fn assert_health_payload(snapshot: &ServiceApiSnapshot) {
    let health = render_service_api_endpoint_response(snapshot, "GET", "/healthz", "");
    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(health.body.as_str()).expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");
}

fn assert_send_payload(snapshot: &ServiceApiSnapshot) {
    let send = render_service_api_endpoint_response(
        snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"serde\"}",
    );
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(send.body.as_str()).expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert!(send_payload.message_id.starts_with("msg-local-"));
}

fn assert_channel_payload(snapshot: &ServiceApiSnapshot) {
    let channel = render_service_api_endpoint_response(
        snapshot,
        "POST",
        "/v1/channels/create",
        "{\"name\":\"alpha\"}",
    );
    let channel_payload: ServiceApiChannelCreateBody =
        parse_service_api_payload(channel.body.as_str())
            .expect("channel payload should deserialize");
    assert_eq!(channel_payload.status, "created");
}

fn assert_task_payload(snapshot: &ServiceApiSnapshot) {
    let task = render_service_api_endpoint_response(
        snapshot,
        "POST",
        "/v1/tasks/create",
        "{\"task\":\"x\"}",
    );
    let task_payload: ServiceApiTaskCreateBody =
        parse_service_api_payload(task.body.as_str()).expect("task payload should deserialize");
    assert_eq!(task_payload.state, "submitted");
}

fn assert_agent_payload(snapshot: &ServiceApiSnapshot) {
    let agent = render_service_api_endpoint_response(
        snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    let agent_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(agent.body.as_str()).expect("agent payload should deserialize");
    assert_eq!(agent_payload.did, "kamn:did:agent:alpha");
    assert_eq!(agent_payload.reputation_score, 500);
    let agent_json: Value =
        serde_json::from_str(agent.body.as_str()).expect("agent payload should parse as json");
    assert_eq!(agent_json["agent_type"], "service-agent");
    assert_eq!(agent_json["model_family"], "service-api");
    assert_eq!(
        agent_json["capabilities"],
        serde_json::json!(["profile:read"])
    );
}
