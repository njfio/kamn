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

#[test]
fn runtime_module_extraction_contract_declares_runtime_state_divergence_module() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        runtime_rs.contains("mod runtime_state_divergence;"),
        "runtime.rs should declare extracted runtime_state_divergence module"
    );
}

#[test]
fn runtime_module_extraction_contract_moves_state_divergence_types_out_of_runtime_rs() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        !runtime_rs.contains("pub enum StateDivergenceStatus {"),
        "runtime.rs should not keep inline StateDivergenceStatus definition"
    );
    assert!(
        !runtime_rs.contains("pub enum StateDivergenceError {"),
        "runtime.rs should not keep inline StateDivergenceError definition"
    );
    assert!(
        !runtime_rs.contains("pub struct StateDivergenceEvaluator;"),
        "runtime.rs should not keep inline StateDivergenceEvaluator definition"
    );
}

#[test]
fn runtime_module_extraction_contract_keeps_state_divergence_impls_in_new_module() {
    let runtime_state_divergence_rs = read_repo_file("runtime_state_divergence.rs");
    assert!(
        runtime_state_divergence_rs.contains("pub enum StateDivergenceStatus {"),
        "runtime_state_divergence module should own StateDivergenceStatus"
    );
    assert!(
        runtime_state_divergence_rs.contains("pub enum StateDivergenceError {"),
        "runtime_state_divergence module should own StateDivergenceError"
    );
    assert!(
        runtime_state_divergence_rs.contains("pub struct StateDivergenceEvaluator;"),
        "runtime_state_divergence module should own StateDivergenceEvaluator"
    );
}

#[test]
fn runtime_module_extraction_contract_declares_runtime_phase_coordination_module() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        runtime_rs.contains("mod runtime_phase_coordination;"),
        "runtime.rs should declare extracted runtime_phase_coordination module"
    );
}

#[test]
fn runtime_module_extraction_contract_moves_phase_coordination_types_out_of_runtime_rs() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        !runtime_rs.contains("pub struct ConstructLockGuard {"),
        "runtime.rs should not keep inline ConstructLockGuard definition"
    );
    assert!(
        !runtime_rs.contains("pub struct ListenerQuorumEvaluator {"),
        "runtime.rs should not keep inline ListenerQuorumEvaluator definition"
    );
    assert!(
        !runtime_rs.contains("pub struct ApproverQuorumEvaluator {"),
        "runtime.rs should not keep inline ApproverQuorumEvaluator definition"
    );
}

#[test]
fn runtime_module_extraction_contract_keeps_phase_coordination_impls_in_new_module() {
    let runtime_phase_coordination_rs = read_repo_file("runtime_phase_coordination.rs");
    assert!(
        runtime_phase_coordination_rs.contains("pub struct ConstructLockGuard {"),
        "runtime_phase_coordination module should own ConstructLockGuard"
    );
    assert!(
        runtime_phase_coordination_rs.contains("pub struct ListenerQuorumEvaluator {"),
        "runtime_phase_coordination module should own ListenerQuorumEvaluator"
    );
    assert!(
        runtime_phase_coordination_rs.contains("pub struct ApproverQuorumEvaluator {"),
        "runtime_phase_coordination module should own ApproverQuorumEvaluator"
    );
}

#[test]
fn runtime_module_extraction_contract_declares_runtime_transport_coordination_module() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        runtime_rs.contains("mod runtime_transport_coordination;"),
        "runtime.rs should declare extracted runtime_transport_coordination module"
    );
}

#[test]
fn runtime_module_extraction_contract_moves_transport_coordination_types_out_of_runtime_rs() {
    let runtime_rs = read_repo_file("runtime.rs");
    assert!(
        !runtime_rs.contains("pub struct WatchdogAnomalyWatchInput {"),
        "runtime.rs should not keep inline WatchdogAnomalyWatchInput definition"
    );
    assert!(
        !runtime_rs.contains("pub struct DeterministicNetworkFaultSimulator {"),
        "runtime.rs should not keep inline DeterministicNetworkFaultSimulator definition"
    );
    assert!(
        !runtime_rs.contains("pub enum NetworkFaultSimulationError {"),
        "runtime.rs should not keep inline NetworkFaultSimulationError definition"
    );
}

#[test]
fn runtime_module_extraction_contract_keeps_transport_coordination_impls_in_new_module() {
    let runtime_transport_coordination_rs = read_repo_file("runtime_transport_coordination.rs");
    assert!(
        runtime_transport_coordination_rs.contains("pub struct WatchdogAnomalyWatchInput {"),
        "runtime_transport_coordination module should own WatchdogAnomalyWatchInput"
    );
    assert!(
        runtime_transport_coordination_rs
            .contains("pub struct DeterministicNetworkFaultSimulator {"),
        "runtime_transport_coordination module should own DeterministicNetworkFaultSimulator"
    );
    assert!(
        runtime_transport_coordination_rs.contains("pub enum NetworkFaultSimulationError {"),
        "runtime_transport_coordination module should own NetworkFaultSimulationError"
    );
}
