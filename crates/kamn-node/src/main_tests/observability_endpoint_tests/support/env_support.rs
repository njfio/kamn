use super::super::*;

pub(in super::super) fn parse_args_with_clean_daemon_env(
    args: Vec<String>,
) -> Result<crate::NodeCli, ConfigError> {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _daemon_control_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_CONTROL", None);
    let _daemon_lifecycle_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_LIFECYCLE_EVENT", None);
    let _max_ticks_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_MAX_TICKS", None);
    let _tick_interval_guard = EnvVarGuard::set("KAMN_NODE_DAEMON_TICK_INTERVAL_MS", None);
    parse_args(args)
}
