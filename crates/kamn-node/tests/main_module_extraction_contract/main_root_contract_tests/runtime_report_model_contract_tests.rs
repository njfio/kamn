use crate::support::{assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_removes_inline_runtime_report_model_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            (
                "struct NodeCli {",
                "main.rs should not keep inline NodeCli struct",
            ),
            (
                "struct PlanningExecution {",
                "main.rs should not keep inline planning execution struct",
            ),
            (
                "struct RecoveryExecution {",
                "main.rs should not keep inline recovery execution struct",
            ),
            (
                "struct DaemonExecution {",
                "main.rs should not keep inline daemon execution struct",
            ),
            (
                "struct DaemonRuntimeOptions {",
                "main.rs should not keep inline daemon runtime options struct",
            ),
            (
                "struct KolmeLiveExecution {",
                "main.rs should not keep inline kolme live execution struct",
            ),
            (
                "struct RuntimeExecutionBundle {",
                "main.rs should not keep inline runtime execution bundle struct",
            ),
            (
                "struct NodeBootstrapReport {",
                "main.rs should not keep inline node bootstrap report struct",
            ),
        ],
    );
}
