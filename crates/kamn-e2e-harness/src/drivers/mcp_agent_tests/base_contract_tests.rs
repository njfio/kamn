use super::*;
use crate::drivers::shared_helpers::live_s07_probe_agent_suffix;

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
    with_env_vars(&[(MCP_AGENT_LIVE_ENV, Some("1"))], || {
        assert!(live_execution_enabled_from_env());
    });
    with_env_vars(&[(MCP_AGENT_LIVE_ENV, Some("0"))], || {
        assert!(!live_execution_enabled_from_env());
    });
}

#[test]
fn unit_mcp_agent_driver_debug_includes_mode_and_live_toggle() {
    let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, false, || Ok(()))
        .expect("driver should build");
    let debug = format!("{driver:?}");
    assert!(debug.contains("McpAgentDriver"));
    assert!(debug.contains("mode"));
    assert!(debug.contains("live_execution_enabled"));
}

#[test]
fn unit_live_s07_probe_agent_suffix_is_non_empty_numeric() {
    let suffix = live_s07_probe_agent_suffix();
    assert!(!suffix.is_empty(), "suffix should be non-empty");
    assert!(
        suffix.chars().all(|character| character.is_ascii_digit()),
        "suffix should be numeric: {suffix}"
    );
}
