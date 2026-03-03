const SERVICE_ROOT_SOURCE: &str = include_str!("../src/service.rs");
const SERVICE_RS_MAX_LINES: usize = 1700;

#[test]
fn contract_issue_6305_service_root_respects_line_budget() {
    let line_count = SERVICE_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= SERVICE_RS_MAX_LINES,
        "service.rs line budget exceeded: actual={line_count}, max={SERVICE_RS_MAX_LINES}"
    );
}

#[test]
fn contract_issue_6305_service_root_wires_external_test_module() {
    assert!(
        SERVICE_ROOT_SOURCE.contains("#[cfg(test)]")
            && SERVICE_ROOT_SOURCE.contains("#[path = \"service_tests.rs\"]")
            && SERVICE_ROOT_SOURCE.contains("mod tests;"),
        "service.rs must wire #[cfg(test)] #[path = \"service_tests.rs\"] mod tests;"
    );
}
