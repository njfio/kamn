const OBSERVABILITY_ENDPOINT_ROOT_SOURCE: &str = include_str!("../src/observability_endpoint.rs");

#[test]
fn regression_observability_endpoint_root_declares_focused_submodules() {
    assert!(
        OBSERVABILITY_ENDPOINT_ROOT_SOURCE.contains("mod endpoint_server;"),
        "observability endpoint root must declare endpoint_server submodule"
    );
    assert!(
        OBSERVABILITY_ENDPOINT_ROOT_SOURCE.contains("mod payload_contract;"),
        "observability endpoint root must declare payload_contract submodule"
    );
    assert!(
        OBSERVABILITY_ENDPOINT_ROOT_SOURCE.contains("mod payload_render;"),
        "observability endpoint root must declare payload_render submodule"
    );
    assert!(
        OBSERVABILITY_ENDPOINT_ROOT_SOURCE.contains("mod tls_mode;"),
        "observability endpoint root must declare tls_mode submodule"
    );
}

#[test]
fn regression_observability_endpoint_root_respects_monolith_line_budget() {
    let line_count = OBSERVABILITY_ENDPOINT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= 1_000,
        "observability endpoint root should remain <= 1000 lines, found {line_count}"
    );
}
