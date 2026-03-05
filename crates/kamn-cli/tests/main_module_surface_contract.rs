use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c10_main_contract_delegates_lib_tests_to_dedicated_module() {
    let lib_rs = read_repo_file("src/lib.rs");
    assert!(
        lib_rs.contains("#[cfg(test)]"),
        "lib.rs should retain cfg(test) delegation marker"
    );
    assert!(
        lib_rs.contains("mod lib_tests;"),
        "lib.rs should delegate tests through lib_tests module"
    );
    let lib_tests_rs = read_repo_file("src/lib_tests.rs");
    assert!(
        lib_tests_rs.contains("fn unit_cli_parser_honors_endpoint_flag()"),
        "lib_tests module should carry existing unit coverage"
    );
}

#[test]
fn spec_c11_main_contract_removes_inline_tests_from_lib() {
    let lib_rs = read_repo_file("src/lib.rs");
    for marker in [
        "fn unit_cli_parser_honors_endpoint_flag()",
        "fn regression_issue_6198_cli_parser_accepts_help_flag_as_command()",
        "fn regression_issue_6219_render_help_text_includes_usage_commands_and_flags()",
    ] {
        assert!(
            !lib_rs.contains(marker),
            "lib.rs should not keep inline unit test marker: {marker}"
        );
    }
}

#[test]
fn spec_c12_main_contract_declares_cli_parse_mapping_module_wiring() {
    let lib_rs = read_repo_file("src/lib.rs");
    assert!(
        lib_rs.contains("mod cli_parse_mapping;"),
        "lib.rs should declare cli_parse_mapping module"
    );
    assert!(
        lib_rs.contains("parse_output_format(raw)"),
        "lib.rs should delegate OutputFormat::parse through parse mapping module"
    );
    assert!(
        lib_rs.contains("parse_command_kind(raw)"),
        "lib.rs should delegate CommandKind::parse through parse mapping module"
    );
}

#[test]
fn spec_c13_main_contract_removes_inline_parse_matches_from_lib() {
    let lib_rs = read_repo_file("src/lib.rs");
    for marker in [
        "\"json\" => Ok(Self::Json)",
        "\"help\" | \"--help\" | \"-h\" => Ok(Self::Help)",
        "\"health\" => Ok(Self::Health)",
    ] {
        assert!(
            !lib_rs.contains(marker),
            "lib.rs should not keep inline parse marker: {marker}"
        );
    }
    let parse_mapping_rs = read_repo_file("src/cli_parse_mapping.rs");
    assert!(
        parse_mapping_rs.contains("pub(super) fn parse_output_format("),
        "parse mapping module should expose output-format parse entrypoint"
    );
    assert!(
        parse_mapping_rs.contains("pub(super) fn parse_command_kind("),
        "parse mapping module should expose command-kind parse entrypoint"
    );
}

#[test]
fn spec_c14_main_contract_declares_cli_models_module_wiring() {
    let lib_rs = read_repo_file("src/lib.rs");
    assert!(
        lib_rs.contains("mod cli_models;"),
        "lib.rs should declare cli_models module"
    );
    assert!(
        lib_rs.contains(
            "pub use cli_models::{CommandKind, CommandOutput, OutputFormat, ParsedCliArgs};"
        ),
        "lib.rs should re-export model types from cli_models module"
    );
}

#[test]
fn spec_c15_main_contract_removes_inline_model_type_definitions_from_lib() {
    let lib_rs = read_repo_file("src/lib.rs");
    for marker in [
        "pub enum OutputFormat {",
        "pub enum CommandKind {",
        "pub struct ParsedCliArgs {",
        "pub struct CommandOutput {",
    ] {
        assert!(
            !lib_rs.contains(marker),
            "lib.rs should not keep inline model type marker: {marker}"
        );
    }
    let cli_models_rs = read_repo_file("src/cli_models.rs");
    assert!(
        cli_models_rs.contains("pub enum OutputFormat"),
        "cli_models module should define OutputFormat"
    );
    assert!(
        cli_models_rs.contains("pub enum CommandKind"),
        "cli_models module should define CommandKind"
    );
    assert!(
        cli_models_rs.contains("pub struct ParsedCliArgs"),
        "cli_models module should define ParsedCliArgs"
    );
    assert!(
        cli_models_rs.contains("pub struct CommandOutput"),
        "cli_models module should define CommandOutput"
    );
}
