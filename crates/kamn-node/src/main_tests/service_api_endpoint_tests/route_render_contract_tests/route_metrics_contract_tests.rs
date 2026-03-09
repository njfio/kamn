use super::super::*;
use crate::service_api_endpoint::{ServiceApiEndpointResponse, ServiceApiSnapshot};

#[path = "route_metrics_contract_tests/support.rs"]
mod support;

const ROUTE_METRIC_CONTRACT_MARKERS: [&str; 3] = [
    "kamn_service_api_route_authz_matrix_total_route_count {}",
    "kamn_service_api_scope_policy_fixture_unique_allow_route_count",
    "kamn_service_api_websocket_reason_taxonomy_info",
];

pub(crate) fn render_metrics_response(snapshot: &ServiceApiSnapshot) -> ServiceApiEndpointResponse {
    render_service_api_endpoint_response(snapshot, "GET", "/metrics", "")
}

pub(crate) fn websocket_upgrade_required_reason_code() -> &'static str {
    "service_api_websocket_upgrade_required"
}

pub(crate) fn assert_common_route_metrics(metrics: &str) {
    support::assert_common_route_metrics(metrics);
}

fn metric_snapshot() -> ServiceApiSnapshot {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34052".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    build_service_api_snapshot(&report)
}

#[test]
fn functional_service_api_endpoint_route_metrics_publish_taxonomy_contracts() {
    let snapshot = metric_snapshot();
    let metrics_response = render_metrics_response(&snapshot);
    assert_eq!(metrics_response.status_code, 200);
    let metrics = metrics_response.body;
    assert_eq!(ROUTE_METRIC_CONTRACT_MARKERS.len(), 3);
    for marker in [
        "kamn_service_api_observability_source{source=\"unknown\"} 1",
        "kamn_service_api_observability_health{health=\"unknown\"} 0",
        "kamn_service_api_observability_latency_p50_ms 0",
    ] {
        assert!(
            metrics.contains(marker),
            "metrics payload missing marker: {marker}"
        );
    }
    assert_common_route_metrics(metrics.as_str());
    assert_websocket_upgrade_required(snapshot);
}

fn assert_websocket_upgrade_required(snapshot: ServiceApiSnapshot) {
    let ws_response = render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(ws_response.status_code, 400);
    let ws_payload = parse_error_envelope(ws_response.body.as_str());
    assert_eq!(ws_payload.error, "bad-request");
    assert_eq!(
        ws_payload.reason_code,
        "service_api_websocket_upgrade_required"
    );
    assert!(ws_payload.message.contains("websocket upgrade required"));
}
