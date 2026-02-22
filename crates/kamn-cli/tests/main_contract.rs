use std::process::Command;

#[test]
fn spec_c07_main_parse_error_contract_exits_with_code_2() {
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
