use std::fs;
use std::path::Path;

#[test]
fn spec_c03_verify_matrix_retains_explicit_error_path_assertions_for_result_surfaces() {
    let parser_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/command_contract.rs");
    let verify_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/command_contract_verify_matrix.rs");

    let parser_content = fs::read_to_string(&parser_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", parser_path.display()));
    let verify_content = fs::read_to_string(&verify_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", verify_path.display()));

    for marker in [
        "parse_command_args([\"run\", \"--mode\", \"sdk-direct\", \"--evidence-dir\"])",
        "expect_err(\"missing evidence-dir value should fail\")",
        "parse_scenario_csv(\"S-01,S-99\")",
        "expect_err(\"unknown scenario should fail\")",
    ] {
        assert!(
            parser_content.contains(marker),
            "missing parser error-path marker in command_contract.rs: {marker}"
        );
    }

    assert!(
        verify_content.contains("execute_verify_contract(&config)"),
        "verify matrix must exercise execute_verify_contract error paths"
    );
    assert!(
        verify_content.matches(".expect_err(").count() >= 15,
        "verify matrix must keep broad explicit error-path assertions for Result surfaces"
    );
}
