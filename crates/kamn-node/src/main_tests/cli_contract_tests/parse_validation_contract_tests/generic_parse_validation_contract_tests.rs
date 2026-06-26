use super::super::*;

#[test]
fn rejects_unknown_argument() {
    let args = with_pairs(approver_args(), &[("--unknown", "")]);
    assert_eq!(
        parse_args(args),
        Err(ConfigError::UnknownArgument("--unknown".to_owned()))
    );
}

#[test]
fn rejects_invalid_output_mode() {
    assert_parse_error(
        with_pairs(approver_args(), &[("--output", "yaml")]),
        ConfigError::InvalidOutputMode("yaml".to_owned()),
    );
}

#[test]
fn rejects_invalid_profile_value() {
    assert_parse_error(
        with_pairs(cli_args(), &[("--profile", "local-unknown")]),
        ConfigError::InvalidNodeProfile("local-unknown".to_owned()),
    );
}

#[test]
fn rejects_invalid_runtime_mode() {
    assert_parse_error(
        with_pairs(processor_runtime_args("service"), &[]),
        ConfigError::InvalidRuntimeMode("service".to_owned()),
    );
}

#[test]
fn rejects_invalid_diagnostics_mode() {
    assert_parse_error(
        with_pairs(
            processor_runtime_args("api"),
            &[("--diagnostics", "extended")],
        ),
        ConfigError::InvalidDiagnosticsMode("extended".to_owned()),
    );
}
