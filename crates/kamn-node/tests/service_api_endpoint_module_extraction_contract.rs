use std::path::Path;
use std::{fs, path::PathBuf};

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn conformance_service_api_endpoint_root_stays_within_line_budget() {
    let root_source = include_str!("../src/service_api_endpoint.rs");
    let root_lines = root_source.lines().count();
    let max_root_lines = std::env::var("KAMN_SERVICE_API_ENDPOINT_ROOT_LINE_BUDGET")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(900);

    assert!(
        root_lines <= max_root_lines,
        "service_api_endpoint.rs root exceeded line budget: {} > {}",
        root_lines,
        max_root_lines
    );
}

#[test]
fn conformance_service_api_endpoint_declares_required_submodules() {
    let root_source = include_str!("../src/service_api_endpoint.rs");
    for marker in [
        "mod auth;",
        "mod middleware_impl;",
        "mod payload;",
        "mod server;",
        "mod websocket;",
    ] {
        assert!(
            root_source.contains(marker),
            "missing service_api_endpoint root module declaration: {marker}"
        );
    }
}

#[test]
fn conformance_service_api_endpoint_submodule_files_exist() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [
        "src/service_api_endpoint/auth.rs",
        "src/service_api_endpoint/middleware_impl.rs",
        "src/service_api_endpoint/payload.rs",
        "src/service_api_endpoint/server.rs",
        "src/service_api_endpoint/websocket.rs",
    ] {
        let full_path = manifest_dir.join(path);
        assert!(
            full_path.exists(),
            "expected service_api_endpoint submodule file missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn conformance_service_api_openapi_spec_contains_required_top_level_markers() {
    let openapi = repo_file("docs/api/service-openapi.yaml");
    for marker in [
        "openapi: 3.1.0",
        "title: KAMN Service API",
        "paths:",
        "components:",
        "securitySchemes:",
    ] {
        assert!(
            openapi.contains(marker),
            "service OpenAPI spec missing top-level marker: {marker}"
        );
    }
}

#[test]
fn conformance_service_api_openapi_spec_covers_required_route_and_auth_markers() {
    let openapi = repo_file("docs/api/service-openapi.yaml");
    for marker in [
        "/healthz:",
        "/v1/messages/send:",
        "/v1/messages/{id}:",
        "/v1/channels/create:",
        "/v1/channels/{id}/messages:",
        "/v1/tasks/create:",
        "/v1/tasks/{id}:",
        "/v1/tasks/{id}/accept:",
        "/v1/tasks/{id}/complete:",
        "/v1/escrow/fund:",
        "/v1/escrow/{id}/release:",
        "/v1/content/register:",
        "/v1/content/{id}:",
        "/v1/content/{id}/expire:",
        "/v1/content/{id}/tombstone:",
        "/v1/bridge/submit:",
        "/v1/bridge/{id}:",
        "/v1/bridge/{id}/forward:",
        "/v1/agents/{did}:",
        "/v1/events/ws:",
        "X-KAMN-Sender-DID",
        "X-KAMN-Request-Nonce",
        "X-KAMN-Request-Signature",
    ] {
        assert!(
            openapi.contains(marker),
            "service OpenAPI spec missing required route/auth marker: {marker}"
        );
    }
}
