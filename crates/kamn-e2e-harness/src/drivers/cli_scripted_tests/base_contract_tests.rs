use super::*;

#[test]
fn unit_parse_bool_flag_accepts_true_like_values() {
    for value in ["1", "true", "TRUE", "yes", "on"] {
        assert!(
            crate::drivers::shared_helpers::parse_bool_flag(value),
            "expected truthy for {value}"
        );
    }
}

#[test]
fn unit_parse_bool_flag_rejects_false_like_values() {
    for value in ["0", "false", "off", "no", ""] {
        assert!(
            !crate::drivers::shared_helpers::parse_bool_flag(value),
            "expected falsey for {value}"
        );
    }
}

#[test]
fn unit_live_execution_enabled_from_env_honors_true_and_false_markers() {
    with_env_vars(&[(CLI_SCRIPTED_LIVE_ENV, Some("1"))], || {
        assert!(live_execution_enabled_from_env());
    });
    with_env_vars(&[(CLI_SCRIPTED_LIVE_ENV, Some("0"))], || {
        assert!(!live_execution_enabled_from_env());
    });
}

#[test]
fn unit_live_s07_probe_agent_suffix_is_non_empty_numeric() {
    let suffix = super::live_s07_probe_agent_suffix();
    assert!(!suffix.is_empty(), "suffix should be non-empty");
    assert!(
        suffix.chars().all(|character| character.is_ascii_digit()),
        "suffix should be numeric: {suffix}",
    );
}

#[test]
fn unit_cli_scripted_driver_debug_includes_live_toggle_field() {
    let driver = CliScriptedDriver::with_runner(false, || Ok(()));
    let debug = format!("{driver:?}");
    assert!(debug.contains("CliScriptedDriver"));
    assert!(debug.contains("live_execution_enabled"));
}
