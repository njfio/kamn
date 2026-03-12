use kamn_core::ConfigError;

use super::super::command::parse_kolme_live_managed_signer_command_spec;

#[test]
fn unit_managed_signer_command_parser_groups_double_quoted_tokens() {
    let parsed = parse_kolme_live_managed_signer_command_spec("printf \"hello world\"")
        .expect("double-quoted token should parse");
    assert_eq!(parsed.executable, "printf");
    assert_eq!(parsed.args, vec!["hello world"]);
}

#[test]
fn regression_managed_signer_command_parser_supports_escaped_double_quote_tokens() {
    let parsed = parse_kolme_live_managed_signer_command_spec("printf \"a\\\"b\"")
        .expect("escaped double quote token should parse");
    assert_eq!(parsed.executable, "printf");
    assert_eq!(parsed.args, vec!["a\"b"]);
}

#[test]
fn regression_managed_signer_command_parser_supports_unquoted_backslash_whitespace_escape() {
    let parsed = parse_kolme_live_managed_signer_command_spec("printf hello\\ world")
        .expect("unquoted backslash whitespace escape should parse");
    assert_eq!(parsed.executable, "printf");
    assert_eq!(parsed.args, vec!["hello world"]);
}

#[test]
fn regression_managed_signer_command_parser_rejects_unterminated_single_quote() {
    let error = parse_kolme_live_managed_signer_command_spec("printf 'unterminated")
        .expect_err("unterminated single quote must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("unterminated quoting or escaping")),
        "unterminated single quote failure must preserve deterministic reason marker"
    );
}

#[test]
fn regression_managed_signer_command_parser_rejects_unterminated_double_quote() {
    let error = parse_kolme_live_managed_signer_command_spec("printf \"unterminated")
        .expect_err("unterminated double quote must fail closed");
    assert!(
        matches!(error, ConfigError::RuntimeKolmeLive(message) if message.contains("unterminated quoting or escaping")),
        "unterminated double quote failure must preserve deterministic reason marker"
    );
}
