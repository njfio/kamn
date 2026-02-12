use kamn_kolme::{
    compose_finality_status_path, compose_notifications_websocket_url,
    is_valid_finality_base_url_input, is_valid_finality_status_path_input,
    is_valid_live_provider_base_url_input, is_valid_live_provider_submit_path_input,
    parse_http_endpoint, parse_websocket_endpoint, KolmeEndpointPolicyError, KolmeHttpScheme,
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
fn functional_compose_finality_status_path_encodes_commit_id_and_separator() {
    assert_eq!(
        compose_finality_status_path("/runtime-commit/status", "kolme-commit:ab12cd34:h42")
            .expect("status path should compose"),
        "/runtime-commit/status?commit_id=kolme-commit%3Aab12cd34%3Ah42"
    );
    assert_eq!(
        compose_finality_status_path(
            "/runtime-commit/status?provider=kolme-fork-local",
            "kolme-commit:ff00"
        )
        .expect("status path should append query"),
        "/runtime-commit/status?provider=kolme-fork-local&commit_id=kolme-commit%3Aff00"
    );
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

#[test]
fn functional_endpoint_policy_accepts_non_empty_finality_checker_inputs() {
    assert!(is_valid_finality_base_url_input("https://kolme.example"));
    assert!(is_valid_finality_status_path_input(
        "/runtime-commit/status"
    ));
}

#[test]
fn regression_issue_1864_endpoint_policy_rejects_empty_finality_checker_inputs() {
    // Regression: #1864
    assert!(!is_valid_finality_base_url_input("   "));
    assert!(!is_valid_finality_status_path_input(""));
}

#[test]
fn functional_endpoint_policy_accepts_non_empty_live_provider_inputs() {
    assert!(is_valid_live_provider_base_url_input(
        "https://kolme.example"
    ));
    assert!(is_valid_live_provider_submit_path_input(
        "/broadcast/runtime-commit"
    ));
}

#[test]
fn regression_issue_1872_endpoint_policy_rejects_empty_live_provider_inputs() {
    // Regression: #1872
    assert!(!is_valid_live_provider_base_url_input(""));
    assert!(!is_valid_live_provider_submit_path_input("   "));
}
