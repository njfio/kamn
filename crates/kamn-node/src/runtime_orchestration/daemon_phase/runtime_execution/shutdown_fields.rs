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
