use crate::support::{assert_contains_all, assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_declares_signer_and_wire_modules() {
    let main_rs = read_repo_file("src/main.rs");
    assert_contains_all(
        &main_rs,
        &[
            ("mod signer;", "main.rs should declare signer module"),
            (
                "mod wire_payload;",
                "main.rs should declare wire_payload module",
            ),
            (
                "mod report_render;",
                "main.rs should declare report_render module",
            ),
            (
                "mod report_builder;",
                "main.rs should declare report_builder module",
            ),
            (
                "mod runtime_kolme_live;",
                "main.rs should declare runtime_kolme_live module",
            ),
            (
                "mod runtime_orchestration;",
                "main.rs should declare runtime_orchestration module",
            ),
            (
                "mod runtime_entrypoint;",
                "main.rs should declare runtime_entrypoint module",
            ),
            ("mod output_io;", "main.rs should declare output_io module"),
            (
                "mod runtime_modes;",
                "main.rs should declare runtime_modes module",
            ),
            (
                "mod runtime_models;",
                "main.rs should declare runtime_models module",
            ),
            (
                "mod main_tests;",
                "main.rs should declare sidecar test module for maintainability",
            ),
        ],
    );
    assert_not_contains_all(
        &main_rs,
        &[(
            "mod tests {",
            "main.rs should not keep inline monolithic tests module",
        )],
    );
}
