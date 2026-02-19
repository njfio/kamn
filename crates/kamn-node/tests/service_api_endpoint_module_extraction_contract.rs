use std::path::Path;

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
