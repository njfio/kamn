use super::support::{
    assert_kolme_live_boundary, assert_kolme_live_impl_boundary, assert_runtime_entrypoint_boundary,
    assert_runtime_mode_delegation, read_runtime_file,
};
use crate::support::read_repo_file;

#[test]
fn main_module_extraction_contract_runtime_module_boundary_parity_markers_remain_stable() {
    let main_rs = read_repo_file("src/main.rs");
    let runtime_orchestration_rs = read_runtime_file("runtime_orchestration.rs");
    let runtime_mode_handlers_rs = read_runtime_file("runtime_orchestration/runtime_mode_handlers.rs");
    let daemon_phase_rs = read_runtime_file("runtime_orchestration/daemon_phase.rs");
    let runtime_kolme_live_rs = read_repo_file("src/runtime_kolme_live.rs");

    assert_runtime_entrypoint_boundary(&main_rs, &runtime_orchestration_rs);
    assert_runtime_mode_delegation(
        &runtime_orchestration_rs,
        &runtime_mode_handlers_rs,
        &daemon_phase_rs,
    );
    assert_kolme_live_boundary(&runtime_orchestration_rs, &runtime_mode_handlers_rs);
    assert_kolme_live_impl_boundary(&runtime_orchestration_rs, &runtime_kolme_live_rs);
}
