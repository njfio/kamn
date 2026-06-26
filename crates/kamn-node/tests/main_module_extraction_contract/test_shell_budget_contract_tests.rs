use crate::support::{count_lines_with_prefix, line_count, read_repo_file};

#[test]
fn main_module_extraction_contract_main_tests_decomposition_and_budget_markers_remain_stable() {
    let main_tests_rs = read_repo_file("src/main_tests.rs");
    assert!(
        main_tests_rs.contains(
            "main_tests structural budget shell only; keep domain tests in src/main_tests/*.rs"
        ),
        "main_tests.rs should carry explicit decomposition drift guard marker"
    );
    assert!(!main_tests_rs.contains("#[test]"));
    assert!(line_count(&main_tests_rs) <= 260);
    assert!(count_lines_with_prefix(&main_tests_rs, "mod ") >= 9);
}

#[test]
fn main_module_extraction_contract_runtime_tests_decomposition_shell_markers_remain_stable() {
    let runtime_tests_rs = read_repo_file("src/main_tests/runtime_tests.rs");
    assert!(
        runtime_tests_rs.contains("runtime_tests structural budget shell only"),
        "runtime_tests.rs should carry explicit decomposition drift guard marker"
    );
    assert!(!runtime_tests_rs.contains("#[test]"));
    assert!(line_count(&runtime_tests_rs) <= 120);
    assert!(count_lines_with_prefix(&runtime_tests_rs, "include!(\"runtime_tests/") >= 6);
}

#[test]
fn main_module_extraction_contract_daemon_tests_decomposition_shell_markers_remain_stable() {
    let daemon_tests_rs = read_repo_file("src/main_tests/daemon_tests.rs");
    assert!(
        daemon_tests_rs.contains(
            "daemon_tests structural budget shell phase3; route runtime/matrix/topology contracts"
        ),
        "daemon_tests.rs should carry explicit phase3 decomposition drift guard marker"
    );
    assert!(daemon_tests_rs.contains("include!(\"daemon_tests/runtime_contract_tests.rs\");"));
    assert!(daemon_tests_rs
        .contains("include!(\"daemon_tests/live_postgres_matrix_contract_tests.rs\");"));
    assert!(daemon_tests_rs
        .contains("include!(\"daemon_tests/live_postgres_topology_contract_tests.rs\");"));
    assert!(daemon_tests_rs.contains(
        "include!(\"daemon_tests/live_postgres_distributed_execution_contract_tests.rs\");"
    ));
    assert!(!daemon_tests_rs.contains("fn functional_runtime_daemon_live_postgres_validation_slice_parallel_lane_topology_scope_contract_is_canonical("));
    assert!(!daemon_tests_rs
        .contains("fn functional_runtime_daemon_emits_structured_transition_markers("));
    assert!(!daemon_tests_rs.contains("fn functional_runtime_daemon_live_postgres_validation_slice_matrix_projection_contract_is_canonical("));
    assert!(line_count(&daemon_tests_rs) <= 300);
    assert!(count_lines_with_prefix(&daemon_tests_rs, "include!(\"daemon_tests/") >= 4);
}
