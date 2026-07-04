pub(crate) type ShutdownArgsSpec = (&'static str, &'static str, &'static str);

fn daemon_base_args(
    role: &'static str,
    max_ticks: &'static str,
    tick_interval_ms: &'static str,
) -> Vec<String> {
    vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        role.to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        max_ticks.to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        tick_interval_ms.to_owned(),
    ]
}

fn extend_shutdown_args(args: &mut Vec<String>, shutdown: ShutdownArgsSpec) {
    let (signal_tick, drain_ticks, timeout_ticks) = shutdown;
    args.extend([
        "--daemon-shutdown-signal-tick".to_owned(),
        signal_tick.to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        drain_ticks.to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        timeout_ticks.to_owned(),
    ]);
}

pub(crate) fn daemon_args_for_live_postgres_profile(
    role: &'static str,
    max_ticks: &'static str,
    tick_interval_ms: &'static str,
    shutdown: Option<ShutdownArgsSpec>,
) -> Vec<String> {
    let mut args = daemon_base_args(role, max_ticks, tick_interval_ms);
    if let Some(spec) = shutdown {
        extend_shutdown_args(&mut args, spec)
    }
    args
}
