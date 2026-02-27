use std::process::Command;

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
