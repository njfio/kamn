use std::fs;
use std::path::Path;

#[test]
fn spec_c03_verify_matrix_retains_explicit_error_path_assertions_for_result_surfaces() {
    let parser_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/command_contract/parser_verify_contract_tests.rs");
    let verify_helper_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/command_contract_verify_matrix/support_helpers.rs");
    let verify_matrix_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/command_contract_verify_matrix");

    let parser_content = fs::read_to_string(&parser_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", parser_path.display()));
    let verify_helper_content = fs::read_to_string(&verify_helper_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", verify_helper_path.display()));
    let verify_matrix_content = read_verify_matrix_sources(&verify_matrix_dir);

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
        verify_helper_content
            .contains("execute_verify_contract(&verify_config(paths)).expect_err(failure_message)"),
        "verify matrix helper must exercise execute_verify_contract error paths"
    );
    assert!(
        verify_matrix_content
            .matches("    expect_verify_failure(")
            .count()
            >= 8,
        "verify matrix must keep broad explicit failure assertions for Result surfaces"
    );
}

fn read_verify_matrix_sources(dir: &Path) -> String {
    let mut content = String::new();
    collect_rust_sources(dir, &mut content);
    content
}

fn collect_rust_sources(dir: &Path, content: &mut String) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("read_dir failed: {error}")) {
        let path = entry.expect("dir entry should be readable").path();
        if path.is_dir() {
            collect_rust_sources(&path, content);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            content.push_str(
                &fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display())),
            );
        }
    }
}
