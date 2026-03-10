use super::support::{
    assert_runtime_orchestration_root_contracts, assert_runtime_submodules, read_runtime_file,
};

#[test]
fn runtime_orchestration_module_extraction_contract_declares_daemon_phase_module() {
    let runtime_orchestration_rs = read_runtime_file("runtime_orchestration.rs");
    let daemon_phase_rs = read_runtime_file("runtime_orchestration/daemon_phase.rs");
    let full_supervisor_rs = read_runtime_file("runtime_orchestration/full_supervisor.rs");
    let runtime_policy_contracts_rs =
        read_runtime_file("runtime_orchestration/runtime_policy_contracts.rs");
    let runtime_mode_handlers_rs =
        read_runtime_file("runtime_orchestration/runtime_mode_handlers.rs");

    assert_runtime_orchestration_root_contracts(&runtime_orchestration_rs);
    assert_runtime_submodules(
        &daemon_phase_rs,
        &full_supervisor_rs,
        &runtime_policy_contracts_rs,
        &runtime_mode_handlers_rs,
    );
}
