use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/src/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read src/{path}: {error}");
    })
}

#[test]
fn runtime_module_extraction_contract_declares_runtime_backpressure_module() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        runtime_rs.contains("mod runtime_backpressure;"),
        "runtime.rs should declare extracted runtime_backpressure module"
    );
}

#[test]
fn runtime_module_extraction_contract_moves_backpressure_types_out_of_runtime_rs() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        !runtime_rs.contains("pub struct RuntimeBackpressurePolicy {"),
        "runtime.rs should not keep inline RuntimeBackpressurePolicy definition"
    );
    assert!(
        !runtime_rs.contains("pub enum RuntimeBackpressureError {"),
        "runtime.rs should not keep inline RuntimeBackpressureError definition"
    );
    assert!(
        !runtime_rs.contains("pub struct DeterministicBackpressureController {"),
        "runtime.rs should not keep inline DeterministicBackpressureController definition"
    );
}

#[test]
fn runtime_module_extraction_contract_keeps_backpressure_impls_in_new_module() {
    let runtime_backpressure_rs = read_repo_file("runtime_backpressure.rs");
    assert!(
        runtime_backpressure_rs.contains("pub struct RuntimeBackpressurePolicy {"),
        "runtime_backpressure module should own RuntimeBackpressurePolicy"
    );
    assert!(
        runtime_backpressure_rs.contains("pub enum RuntimeBackpressureError {"),
        "runtime_backpressure module should own RuntimeBackpressureError"
    );
    assert!(
        runtime_backpressure_rs.contains("pub struct DeterministicBackpressureController {"),
        "runtime_backpressure module should own DeterministicBackpressureController"
    );
}
