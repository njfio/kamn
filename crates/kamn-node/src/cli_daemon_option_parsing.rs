use super::{
    cli_value_parsers::{parse_daemon_control_arg, parse_daemon_lifecycle_event},
    ConfigError, PeerLifecycleEvent,
};

pub(super) struct DaemonOptionState<'a> {
    pub(super) daemon_max_ticks: &'a mut Option<u64>,
    pub(super) daemon_tick_interval_ms: &'a mut Option<u64>,
    pub(super) daemon_shutdown_signal_ticks: &'a mut Vec<u64>,
    pub(super) daemon_shutdown_os_signals: &'a mut bool,
    pub(super) daemon_shutdown_drain_ticks: &'a mut Option<u64>,
    pub(super) daemon_shutdown_timeout_ticks: &'a mut Option<u64>,
    pub(super) daemon_peer_id: &'a mut Option<String>,
    pub(super) daemon_lifecycle_events: &'a mut Vec<PeerLifecycleEvent>,
}

pub(super) fn try_parse_daemon_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut DaemonOptionState<'_>,
) -> Result<bool, ConfigError> {
    if try_parse_daemon_tick_option(arg, iter, state)? {
        return Ok(true);
    }
    if try_parse_daemon_shutdown_option(arg, iter, state)? {
        return Ok(true);
    }
    try_parse_daemon_peer_lifecycle_option(arg, iter, state)
}

fn try_parse_daemon_tick_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut DaemonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--daemon-max-ticks" => {
            set_optional_numeric(iter, "--daemon-max-ticks", state.daemon_max_ticks).map(|_| true)
        }
        "--daemon-tick-interval-ms" => set_optional_numeric(
            iter,
            "--daemon-tick-interval-ms",
            state.daemon_tick_interval_ms,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn try_parse_daemon_shutdown_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut DaemonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--daemon-shutdown-signal-tick" => push_numeric_value(
            iter,
            "--daemon-shutdown-signal-tick",
            state.daemon_shutdown_signal_ticks,
        )
        .map(|_| true),
        "--daemon-shutdown-os-signals" => {
            *state.daemon_shutdown_os_signals = true;
            Ok(true)
        }
        "--daemon-shutdown-drain-ticks" => set_optional_numeric(
            iter,
            "--daemon-shutdown-drain-ticks",
            state.daemon_shutdown_drain_ticks,
        )
        .map(|_| true),
        "--daemon-shutdown-timeout-ticks" => set_optional_numeric(
            iter,
            "--daemon-shutdown-timeout-ticks",
            state.daemon_shutdown_timeout_ticks,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn try_parse_daemon_peer_lifecycle_option(
    arg: &str,
    iter: &mut std::vec::IntoIter<String>,
    state: &mut DaemonOptionState<'_>,
) -> Result<bool, ConfigError> {
    match arg {
        "--daemon-peer-id" => {
            set_string_option(iter, "--daemon-peer-id", state.daemon_peer_id).map(|_| true)
        }
        "--daemon-lifecycle-event" => parse_lifecycle_event_option(
            iter,
            "--daemon-lifecycle-event",
            state.daemon_lifecycle_events,
        )
        .map(|_| true),
        _ => Ok(false),
    }
}

fn set_optional_numeric(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Option<u64>,
) -> Result<(), ConfigError> {
    *target = Some(parse_numeric_value(iter, flag)?);
    Ok(())
}

fn push_numeric_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Vec<u64>,
) -> Result<(), ConfigError> {
    target.push(parse_numeric_value(iter, flag)?);
    Ok(())
}

fn set_string_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Option<String>,
) -> Result<(), ConfigError> {
    *target = Some(read_required_value(iter, flag)?);
    Ok(())
}

fn parse_lifecycle_event_option(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
    target: &mut Vec<PeerLifecycleEvent>,
) -> Result<(), ConfigError> {
    let value = read_required_value(iter, flag)?;
    target.push(parse_daemon_lifecycle_event(&value)?);
    Ok(())
}

fn parse_numeric_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<u64, ConfigError> {
    let value = read_required_value(iter, flag)?;
    parse_daemon_control_arg(&value)
}

fn read_required_value(
    iter: &mut std::vec::IntoIter<String>,
    flag: &'static str,
) -> Result<String, ConfigError> {
    iter.next().ok_or(ConfigError::MissingArgumentValue(flag))
}
