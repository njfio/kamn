use super::*;

fn daemon_lifecycle_event_as_str(event: PeerLifecycleEvent) -> &'static str {
    match event {
        PeerLifecycleEvent::StartConnect => "start-connect",
        PeerLifecycleEvent::HandshakeSucceeded => "handshake-succeeded",
        PeerLifecycleEvent::HeartbeatMissed => "heartbeat-missed",
        PeerLifecycleEvent::HeartbeatRestored => "heartbeat-restored",
        PeerLifecycleEvent::Disconnect => "disconnect",
        PeerLifecycleEvent::Rejoin => "rejoin",
    }
}

fn peer_lifecycle_state_as_str(state: PeerLifecycleState) -> &'static str {
    match state {
        PeerLifecycleState::Disconnected => "disconnected",
        PeerLifecycleState::Connecting => "connecting",
        PeerLifecycleState::Active => "active",
        PeerLifecycleState::Degraded => "degraded",
    }
}

pub(super) fn daemon_shutdown_drain_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "completed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "timeout"
    } else {
        "not-signaled"
    }
}

pub(super) fn daemon_shutdown_snapshot_flush_status(completion_reason: &str) -> &'static str {
    if completion_reason.starts_with("graceful-shutdown:signal@") {
        "snapshot-flushed"
    } else if completion_reason.starts_with("graceful-shutdown-timeout:signal@") {
        "snapshot-flush-timeout"
    } else {
        "snapshot-not-requested"
    }
}

pub(super) fn daemon_shutdown_signal_tick(completion_reason: &str) -> Option<&str> {
    completion_reason
        .strip_prefix("graceful-shutdown:signal@")
        .or_else(|| completion_reason.strip_prefix("graceful-shutdown-timeout:signal@"))
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .filter(|tick| !tick.is_empty() && tick.chars().all(|c| c.is_ascii_digit()))
}

pub(super) fn daemon_shutdown_reason_field<'a>(
    completion_reason: &'a str,
    key: &str,
) -> Option<&'a str> {
    completion_reason.split(';').find_map(|segment| {
        let (field, value) = segment.split_once('=')?;
        if field == key {
            return Some(value);
        }
        None
    })
}

pub(super) fn execute_daemon_runtime(
    runtime_mode: RuntimeMode,
    execution_id: &str,
    options: DaemonRuntimeOptions,
) -> Result<DaemonExecution, ConfigError> {
    let DaemonRuntimeOptions {
        daemon_max_ticks,
        daemon_tick_interval_ms,
        daemon_shutdown_signal_ticks,
        daemon_shutdown_os_signals,
        daemon_shutdown_drain_ticks,
        daemon_shutdown_timeout_ticks,
        daemon_peer_id,
        daemon_lifecycle_events,
    } = options;
    let max_ticks =
        daemon_max_ticks.ok_or(ConfigError::MissingArgumentValue("--daemon-max-ticks"))?;
    let tick_interval_ms = daemon_tick_interval_ms.ok_or(ConfigError::MissingArgumentValue(
        "--daemon-tick-interval-ms",
    ))?;
    let max_ticks_label = max_ticks.to_string();
    let tick_interval_ms_label = tick_interval_ms.to_string();
    log_info(
        "node.runtime.daemon.execute.start",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("max_ticks", max_ticks_label.as_str()),
            ("tick_interval_ms", tick_interval_ms_label.as_str()),
            ("execution_id", execution_id),
        ],
    )?;
    let (peer_id, peer_lifecycle_final_state, peer_lifecycle_applied_events) = match daemon_peer_id
    {
        Some(peer_id) => {
            let mut lifecycle = PeerLifecycle::new(&peer_id)
                .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
            let mut applied_events = Vec::with_capacity(daemon_lifecycle_events.len());
            for event in daemon_lifecycle_events {
                lifecycle
                    .transition(event)
                    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
                applied_events.push(daemon_lifecycle_event_as_str(event).to_owned());
            }
            (
                Some(peer_id),
                Some(peer_lifecycle_state_as_str(lifecycle.state()).to_owned()),
                Some(applied_events),
            )
        }
        None => (None, None, None),
    };
    let daemon_completion = if should_use_os_signal_shutdown(
        runtime_mode,
        daemon_shutdown_os_signals,
        daemon_shutdown_signal_ticks.as_slice(),
    ) {
        evaluate_daemon_completion_with_os_signals(
            max_ticks,
            tick_interval_ms,
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?
    } else {
        evaluate_daemon_completion(
            max_ticks,
            daemon_shutdown_signal_ticks.as_slice(),
            daemon_shutdown_drain_ticks,
            daemon_shutdown_timeout_ticks,
        )
    };
    let daemon_observability = build_daemon_observability_telemetry(
        tick_interval_ms,
        daemon_completion.completion_reason.as_str(),
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))?;
    let shutdown_drain_status =
        daemon_shutdown_drain_status(daemon_completion.completion_reason.as_str());
    let shutdown_snapshot_flush_status =
        daemon_shutdown_snapshot_flush_status(daemon_completion.completion_reason.as_str());
    let shutdown_signal_tick =
        daemon_shutdown_signal_tick(daemon_completion.completion_reason.as_str()).unwrap_or("none");
    let shutdown_drain_ticks =
        daemon_shutdown_reason_field(daemon_completion.completion_reason.as_str(), "drain_ticks")
            .unwrap_or("0");
    let shutdown_timeout_ticks = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "timeout_ticks",
    )
    .unwrap_or("0");
    let shutdown_ignored_signals = daemon_shutdown_reason_field(
        daemon_completion.completion_reason.as_str(),
        "ignored_signals",
    )
    .unwrap_or("0");
    let executed_ticks_label = daemon_completion.executed_ticks.to_string();
    log_info(
        "node.runtime.daemon.execute.complete",
        &[
            ("runtime_mode", runtime_mode.as_str()),
            ("executed_ticks", executed_ticks_label.as_str()),
            (
                "completion_reason",
                daemon_completion.completion_reason.as_str(),
            ),
            ("shutdown_drain_status", shutdown_drain_status),
            (
                "shutdown_snapshot_flush_status",
                shutdown_snapshot_flush_status,
            ),
            ("shutdown_signal_tick", shutdown_signal_tick),
            ("shutdown_drain_ticks", shutdown_drain_ticks),
            ("shutdown_timeout_ticks", shutdown_timeout_ticks),
            ("shutdown_ignored_signals", shutdown_ignored_signals),
            ("execution_id", execution_id),
        ],
    )?;
    Ok(DaemonExecution {
        max_ticks,
        tick_interval_ms,
        executed_ticks: daemon_completion.executed_ticks,
        completion_reason: daemon_completion.completion_reason,
        observability_latency_p50_ms: daemon_observability.latency_p50_ms,
        observability_latency_p99_ms: daemon_observability.latency_p99_ms,
        observability_throughput_tps: daemon_observability.throughput_tps,
        observability_error_rate_bps: daemon_observability.error_rate_bps,
        observability_availability_bps: daemon_observability.availability_bps,
        observability_health: daemon_observability.health,
        observability_alert_count: daemon_observability.alert_count,
        observability_reason_code: daemon_observability.reason_code,
        observability_transport_checkpoint_failures: daemon_observability
            .transport_checkpoint_failures,
        observability_signer_checkpoint_failures: daemon_observability.signer_checkpoint_failures,
        observability_commit_checkpoint_failures: daemon_observability.commit_checkpoint_failures,
        peer_id,
        peer_lifecycle_final_state,
        peer_lifecycle_applied_events,
    })
}
