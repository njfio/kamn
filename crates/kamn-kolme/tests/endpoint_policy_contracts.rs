use kamn_kolme::{
    compose_notifications_websocket_url, parse_http_endpoint, parse_websocket_endpoint,
    KolmeEndpointPolicyError, KolmeHttpScheme,
};

#[test]
fn functional_parse_http_endpoint_normalizes_path_and_scheme() {
    let endpoint = parse_http_endpoint("https://kolme.example/api", "runtime-commit/status")
        .expect("endpoint should parse");
    assert_eq!(endpoint.scheme, KolmeHttpScheme::Https);
    assert_eq!(endpoint.host, "kolme.example");
    assert_eq!(endpoint.port, 443);
    assert_eq!(endpoint.target_path, "/api/runtime-commit/status");
}

#[test]
fn functional_compose_notifications_websocket_url_maps_https_to_wss() {
    let notifications_url =
        compose_notifications_websocket_url("https://kolme.example/base", "/notifications")
            .expect("notifications url should compose");
    assert_eq!(notifications_url, "wss://kolme.example/base/notifications");
}

#[test]
fn regression_issue_1729_endpoint_validation_fails_closed() {
    // Regression: #1729
    assert_eq!(
        parse_http_endpoint("ftp://kolme.example", "/status"),
        Err(KolmeEndpointPolicyError::Unavailable {
            reason: "base_url scheme must be http:// or https://".to_owned(),
        })
    );

    assert_eq!(
        parse_websocket_endpoint("http://kolme.example/notifications"),
        Err(KolmeEndpointPolicyError::Unavailable {
            reason: "notifications_url scheme must be ws:// or wss://".to_owned(),
        })
    );
}
