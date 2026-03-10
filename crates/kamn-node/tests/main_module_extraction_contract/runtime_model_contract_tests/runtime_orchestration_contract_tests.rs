use crate::support::{assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_removes_inline_runtime_orchestration_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            ("fn execute_daemon_runtime(", "main.rs should not keep inline daemon runtime executor"),
            ("fn classify_full_supervisor_stop_contract_violation(", "main.rs should not keep inline full supervisor stop classifier"),
            ("fn enforce_kolme_live_signer_contract_policy(", "main.rs should not keep inline signer policy enforcement helper"),
            ("fn execute(cli: NodeCli)", "main.rs should delegate runtime execution to runtime_orchestration module"),
        ],
    );
}
