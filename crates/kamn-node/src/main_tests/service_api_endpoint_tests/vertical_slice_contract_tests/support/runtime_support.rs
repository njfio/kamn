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
    let _route_guard = relay_route_guard(recipient_addr, recipient_did);
    let _key_guard = auth_key_guard();
    let report = relay_projection_report();
    assert_eq!(report.runtime_mode, "daemon");
}

fn relay_route_guard(recipient_addr: &str, recipient_did: &str) -> EnvVarGuard {
    let route_map = format!(r#"{{"{recipient_did}":"{recipient_addr}"}}"#);
    EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
        Some(route_map.as_str()),
    )
}

fn auth_key_guard() -> EnvVarGuard {
    EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
    )
}

fn relay_projection_report() -> NodeBootstrapReport {
    let cli = parse_args(vec![
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
    execute(cli).expect("daemon relay projection should succeed")
}
