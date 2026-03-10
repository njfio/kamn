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
    with_env_vars(
        &[
            (SDK_DIRECT_LIVE_ENV, Some("1")),
            ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ],
        || {
            assert!(
                live_execution_enabled_from_env(),
                "truthy env value should enable live SDK-direct mode",
            );
        },
    );

    with_env_vars(&[(SDK_DIRECT_LIVE_ENV, Some("0"))], || {
        assert!(
            !live_execution_enabled_from_env(),
            "falsey env value should disable live SDK-direct mode",
        );
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
fn unit_sdk_direct_driver_debug_includes_live_toggle_field() {
    let driver = SdkDirectDriver::with_probe(false, || Ok(()));
    let debug = format!("{driver:?}");
    assert!(
        debug.contains("SdkDirectDriver"),
        "debug output should include struct name: {debug}",
    );
    assert!(
        debug.contains("live_execution_enabled"),
        "debug output should include live toggle field: {debug}",
    );
}
