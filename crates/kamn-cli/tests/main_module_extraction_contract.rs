use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_main_contract_file_stays_within_size_budget() {
    let main_contract = read_repo_file("tests/main_contract.rs");
    let line_count = main_contract.lines().count();
    assert!(
        line_count <= 200,
        "main_contract.rs should stay within 200-line test budget after split; got {line_count}"
    );
}

#[test]
fn spec_c02_main_contract_file_removes_extraction_marker_blocks() {
    let main_contract = read_repo_file("tests/main_contract.rs");
    for marker in [
        "fn spec_c06_main_contract_declares_cli_args_module_extraction_wiring()",
        "fn spec_c07_main_contract_removes_inline_arg_help_parser_logic_from_lib()",
        "fn spec_c08_main_contract_declares_cli_dispatch_module_extraction_wiring()",
        "fn spec_c09_main_contract_removes_inline_dispatch_logic_from_lib()",
        "fn spec_c10_main_contract_delegates_lib_tests_to_dedicated_module()",
        "fn spec_c11_main_contract_removes_inline_tests_from_lib()",
        "fn spec_c12_main_contract_declares_cli_parse_mapping_module_wiring()",
        "fn spec_c13_main_contract_removes_inline_parse_matches_from_lib()",
        "fn spec_c14_main_contract_declares_cli_models_module_wiring()",
        "fn spec_c15_main_contract_removes_inline_model_type_definitions_from_lib()",
    ] {
        assert!(
            !main_contract.contains(marker),
            "main_contract.rs should not keep extraction marker block: {marker}"
        );
    }
}

#[test]
fn spec_c03_extraction_contract_file_owns_marker_assertion_surface() {
    let module_contract = read_repo_file("tests/main_module_extraction_contract.rs");
    for marker in [
        "fn spec_c01_main_contract_file_stays_within_size_budget()",
        "fn spec_c02_main_contract_file_removes_extraction_marker_blocks()",
    ] {
        assert!(
            module_contract.contains(marker),
            "main_module_extraction_contract should retain marker assertion: {marker}"
        );
    }
}
