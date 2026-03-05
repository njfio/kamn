use std::{fs, process::Command};

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_main_help_flag_contract_exits_with_code_0() {
    let output = Command::new(env!("CARGO_BIN_EXE_kamn-cli"))
        .arg("--help")
        .output()
        .expect("kamn-cli should execute");
    assert_eq!(
        output.status.code(),
        Some(0),
        "help should exit with success code 0",
    );

    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    assert!(
        stdout.contains("Usage:"),
        "help output should include usage text: {stdout}",
    );
}

#[test]
fn spec_c02_main_short_help_flag_contract_exits_with_code_0() {
    let output = Command::new(env!("CARGO_BIN_EXE_kamn-cli"))
        .arg("-h")
        .output()
        .expect("kamn-cli should execute");
    assert_eq!(
        output.status.code(),
        Some(0),
        "short help should exit with success code 0",
    );
}

#[test]
fn spec_c03_main_help_command_contract_exits_with_code_0() {
    let output = Command::new(env!("CARGO_BIN_EXE_kamn-cli"))
        .arg("help")
        .output()
        .expect("kamn-cli should execute");
    assert_eq!(
        output.status.code(),
        Some(0),
        "help command should exit with success code 0",
    );

    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    for marker in ["--endpoint", "--format", "send-message", "health"] {
        assert!(
            stdout.contains(marker),
            "help output should include marker `{marker}`: {stdout}",
        );
    }
}

#[test]
fn spec_c05_main_parse_error_contract_exits_with_code_2() {
    let output = Command::new(env!("CARGO_BIN_EXE_kamn-cli"))
        .output()
        .expect("kamn-cli should execute");
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing command should exit with parse-error code 2",
    );

    let stderr = String::from_utf8_lossy(output.stderr.as_slice());
    assert!(
        stderr.contains("kamn-cli parse error: missing command"),
        "stderr should include parse error marker: {stderr}",
    );
}

#[test]
fn spec_c06_main_contract_declares_cli_args_module_extraction_wiring() {
    let lib_rs = read_repo_file("src/lib.rs");
    assert!(
        lib_rs.contains("mod cli_args;"),
        "lib.rs should declare cli_args module"
    );
    assert!(
        lib_rs.contains("parse_cli_args_impl(args)"),
        "lib.rs should delegate parse_cli_args through cli_args module"
    );
    assert!(
        lib_rs.contains("is_help_request_impl(args)"),
        "lib.rs should delegate is_help_request through cli_args module"
    );
    assert!(
        lib_rs.contains("render_help_text_impl()"),
        "lib.rs should delegate render_help_text through cli_args module"
    );
}

#[test]
fn spec_c07_main_contract_removes_inline_arg_help_parser_logic_from_lib() {
    let lib_rs = read_repo_file("src/lib.rs");
    for marker in [
        "fn env_var_or_default(",
        "let mut index = 0;",
        "while index < args.len() {",
        "let command_list = CLI_SUPPORTED_COMMANDS.join(\", \");",
        "let flags = CLI_HELP_FLAGS.join(\", \");",
    ] {
        assert!(
            !lib_rs.contains(marker),
            "lib.rs should not keep inline arg/help parser marker: {marker}"
        );
    }
    let cli_args_rs = read_repo_file("src/cli_args.rs");
    assert!(
        cli_args_rs.contains("pub(super) fn parse_cli_args_impl<I, S>("),
        "cli_args module should define parse_cli_args implementation entrypoint"
    );
    assert!(
        cli_args_rs.contains("pub(super) fn is_help_request_impl<I, S>("),
        "cli_args module should define is_help_request implementation entrypoint"
    );
    assert!(
        cli_args_rs.contains("pub(super) fn render_help_text_impl() -> String"),
        "cli_args module should define render_help_text implementation entrypoint"
    );
}

#[test]
fn spec_c08_main_contract_declares_cli_dispatch_module_extraction_wiring() {
    let lib_rs = read_repo_file("src/lib.rs");
    assert!(
        lib_rs.contains("mod cli_dispatch;"),
        "lib.rs should declare cli_dispatch module"
    );
    assert!(
        lib_rs.contains("dispatch_impl(parsed)"),
        "lib.rs should delegate dispatch through cli_dispatch module"
    );
}

#[test]
fn spec_c09_main_contract_removes_inline_dispatch_logic_from_lib() {
    let lib_rs = read_repo_file("src/lib.rs");
    for marker in [
        "CommandKind::Help => Ok(help_output())",
        "CommandKind::Register => commands::register::execute(parsed)",
        "CommandKind::Health => commands::health::execute(parsed)",
    ] {
        assert!(
            !lib_rs.contains(marker),
            "lib.rs should not keep inline dispatch marker: {marker}"
        );
    }
    let cli_dispatch_rs = read_repo_file("src/cli_dispatch.rs");
    assert!(
        cli_dispatch_rs.contains("pub(super) fn dispatch_impl("),
        "cli_dispatch module should define dispatch implementation entrypoint"
    );
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
