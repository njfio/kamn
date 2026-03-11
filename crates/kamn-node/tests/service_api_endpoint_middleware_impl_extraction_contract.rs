use std::path::Path;

#[test]
fn middleware_impl_root_stays_within_shell_budget() {
    let root = include_str!("../src/service_api_endpoint/middleware_impl.rs");
    let lines = root.lines().count();
    let max_lines = 180usize;
    assert!(
        lines <= max_lines,
        "middleware_impl.rs exceeded root shell budget: {lines} > {max_lines}"
    );
}

#[test]
fn middleware_impl_root_declares_required_submodules() {
    let root = include_str!("../src/service_api_endpoint/middleware_impl.rs");
    for marker in [
        "mod auth_flow;",
        "mod request_parsing;",
        "mod error_response;",
        "mod http_routes;",
        "mod websocket_routes;",
        "mod payload_parsing;",
        "mod lifecycle_policy;",
    ] {
        assert!(
            root.contains(marker),
            "missing middleware_impl root submodule declaration: {marker}"
        );
    }
}

#[test]
fn middleware_impl_extracted_files_exist_and_stay_bounded() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in [
        "src/service_api_endpoint/middleware_impl/auth_flow.rs",
        "src/service_api_endpoint/middleware_impl/request_parsing.rs",
        "src/service_api_endpoint/middleware_impl/error_response.rs",
        "src/service_api_endpoint/middleware_impl/http_routes.rs",
        "src/service_api_endpoint/middleware_impl/websocket_routes.rs",
        "src/service_api_endpoint/middleware_impl/payload_parsing.rs",
        "src/service_api_endpoint/middleware_impl/lifecycle_policy.rs",
    ] {
        let full = manifest_dir.join(path);
        assert!(full.exists(), "expected extracted middleware file missing: {}", full.display());
        let source = std::fs::read_to_string(&full)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", full.display()));
        let lines = source.lines().count();
        assert!(
            lines <= 200,
            "extracted middleware file should stay within 200 lines: {path} has {lines}"
        );
    }
}

#[test]
fn middleware_impl_root_no_longer_keeps_inline_payload_and_policy_helpers() {
    let root = include_str!("../src/service_api_endpoint/middleware_impl.rs");
    for marker in [
        "fn extract_channel_id_from_payload(",
        "fn extract_canonical_recipient_did_from_payload(",
        "fn parse_relay_ingest_payload(",
        "fn parse_agent_registration_payload(",
        "fn parse_agent_search_payload(",
        "pub(super) fn service_api_lifecycle_rejection_policy(",
        "pub(super) fn emit_service_api_request_outcome(",
    ] {
        assert!(
            !root.contains(marker),
            "middleware_impl root still keeps moved monolith section: {marker}"
        );
    }
}
