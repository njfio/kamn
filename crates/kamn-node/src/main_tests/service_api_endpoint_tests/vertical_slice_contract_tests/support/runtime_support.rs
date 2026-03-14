use super::env_support::{set_relay_spool_env, set_state_file_env};
use super::super::super::*;

pub(crate) fn project_relay_to_recipient(
    sender_state_file: &std::path::Path,
    sender_spool_file: &std::path::Path,
    recipient_addr: &str,
    recipient_did: &str,
) {
    let _state_guard = set_state_file_env(sender_state_file);
    let _spool_guard = set_relay_spool_env(sender_spool_file);
    let route_map = format!(r#"{{"{recipient_did}":"{recipient_addr}"}}"#);
    let _route_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
        Some(route_map.as_str()),
    );
    let _key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
    );
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon relay projection should succeed");
    assert_eq!(report.runtime_mode, "daemon");
}
