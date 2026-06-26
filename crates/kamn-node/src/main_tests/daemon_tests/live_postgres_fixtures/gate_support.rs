use super::constants::*;
use super::*;
pub(crate) fn parse_args_with_clean_daemon_env(
    args: Vec<String>,
) -> Result<crate::NodeCli, ConfigError> {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", None);
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", None);
    parse_args(args)
}

pub(crate) fn live_postgres_url() -> Option<String> {
    let preferred = std::env::var("KAMN_TEST_POSTGRES_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let fallback = std::env::var("DATABASE_URL")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    preferred.or(fallback)
}

pub(crate) fn resolve_live_postgres_gate_decision() -> (&'static str, Option<String>) {
    match live_postgres_url() {
        Some(database_url) => (
            LIVE_POSTGRES_ADAPTER_CONNECTED_REASON_CODE,
            Some(database_url),
        ),
        None => (LIVE_POSTGRES_ENV_UNSET_REASON_CODE, None),
    }
}

pub(crate) fn parse_live_postgres_distributed_host_pair(raw: &str) -> Option<(String, String)> {
    let hosts = raw
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if hosts.len() != 2 || hosts[0] == hosts[1] {
        return None;
    }
    Some((hosts[0].clone(), hosts[1].clone()))
}
