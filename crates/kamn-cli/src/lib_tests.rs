use super::{
    dispatch, is_help_request, parse_cli_args, render_help_text, CommandKind, OutputFormat,
};

fn parse_help_command() -> super::ParsedCliArgs {
    parse_cli_args(["kamn-cli", "--help"]).expect("help command should parse")
}

#[test]
fn unit_cli_parser_honors_endpoint_flag() {
    let parsed = parse_cli_args(["kamn-cli", "health", "--endpoint", "http://localhost:8080"])
        .expect("parsed");
    assert_eq!(parsed.endpoint, "http://localhost:8080");
    assert_eq!(parsed.output_format, OutputFormat::Json);
}

#[test]
fn regression_issue_6198_cli_parser_accepts_help_flag_as_command() {
    let parsed = parse_help_command();
    assert_eq!(parsed.command, CommandKind::Help);
}

#[test]
fn regression_issue_6198_cli_dispatch_renders_usage_surface() {
    let parsed = parse_help_command();
    let output = dispatch(&parsed).expect("help command should dispatch");
    assert!(output.text.contains("usage=kamn-cli <command>"));
    assert!(output.text.contains("commands=register"));
    assert!(output.text.contains("flags=--help"));
    assert!(output.json.contains("\"usage\":\"kamn-cli <command>"));
    assert!(output.json.contains("\"commands\":[\"register\""));
}

#[test]
fn regression_issue_6213_cli_parser_rejects_unknown_long_flag() {
    let error = parse_cli_args(["kamn-cli", "health", "--unknown"]).expect_err("must fail");
    assert_eq!(error, "unsupported flag: --unknown");
}

#[test]
fn regression_issue_6213_cli_parser_rejects_unknown_short_flag() {
    let error = parse_cli_args(["kamn-cli", "health", "-x"]).expect_err("must fail");
    assert_eq!(error, "unsupported flag: -x");
}

#[test]
fn regression_issue_6213_cli_parser_keeps_non_flag_passthrough() {
    let parsed = parse_cli_args(["kamn-cli", "send-message", "payload.json"])
        .expect("non-flag positional arguments should pass through");
    assert_eq!(parsed.passthrough, vec!["payload.json".to_owned()]);
}

#[test]
fn regression_issue_6219_help_request_true_for_long_flag() {
    assert!(is_help_request(["kamn-cli", "--help"]));
}

#[test]
fn regression_issue_6219_help_request_true_for_short_flag() {
    assert!(is_help_request(["kamn-cli", "-h"]));
}

#[test]
fn regression_issue_6219_help_request_false_without_help_flags() {
    assert!(!is_help_request(["kamn-cli", "health"]));
}

#[test]
fn regression_issue_6219_render_help_text_includes_usage_commands_and_flags() {
    let help = render_help_text();
    assert!(help.contains("Usage: kamn-cli <command>"));
    assert!(help.contains("Commands: register"));
    assert!(help.contains("Flags: --help"));
}
