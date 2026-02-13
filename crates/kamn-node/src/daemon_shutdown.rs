#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DaemonCompletion {
    pub(super) executed_ticks: u64,
    pub(super) completion_reason: String,
}

pub(super) fn evaluate_daemon_completion(
    max_ticks: u64,
    shutdown_signal_ticks: &[u64],
    drain_ticks: Option<u64>,
    timeout_ticks: Option<u64>,
) -> DaemonCompletion {
    if shutdown_signal_ticks.is_empty() {
        return DaemonCompletion {
            executed_ticks: max_ticks,
            completion_reason: "tick-budget-exhausted".to_owned(),
        };
    }

    let first_valid_signal_tick = shutdown_signal_ticks
        .iter()
        .copied()
        .filter(|tick| *tick <= max_ticks)
        .min();
    let ignored_signals = match first_valid_signal_tick {
        Some(_) => shutdown_signal_ticks.len().saturating_sub(1),
        None => shutdown_signal_ticks.len(),
    };

    let Some(signal_tick) = first_valid_signal_tick else {
        return DaemonCompletion {
            executed_ticks: max_ticks,
            completion_reason: format!("tick-budget-exhausted;ignored_signals={ignored_signals}"),
        };
    };

    let drain_ticks = drain_ticks.unwrap_or(1);
    let timeout_ticks = timeout_ticks.unwrap_or(1);
    let target_drain_tick = signal_tick.saturating_add(drain_ticks);
    let timeout_deadline_tick = signal_tick.saturating_add(timeout_ticks).min(max_ticks);

    if target_drain_tick <= timeout_deadline_tick && target_drain_tick <= max_ticks {
        return DaemonCompletion {
            executed_ticks: target_drain_tick,
            completion_reason: format!(
                "graceful-shutdown:signal@{signal_tick};drain_ticks={drain_ticks};timeout_ticks={timeout_ticks};ignored_signals={ignored_signals}"
            ),
        };
    }

    DaemonCompletion {
        executed_ticks: timeout_deadline_tick,
        completion_reason: format!(
            "graceful-shutdown-timeout:signal@{signal_tick};drain_ticks={drain_ticks};timeout_ticks={timeout_ticks};ignored_signals={ignored_signals}"
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate_daemon_completion;

    #[test]
    fn unit_daemon_completion_defaults_to_tick_budget_without_shutdown_signal() {
        let completion = evaluate_daemon_completion(6, &[], None, None);
        assert_eq!(completion.executed_ticks, 6);
        assert_eq!(completion.completion_reason, "tick-budget-exhausted");
    }

    #[test]
    fn unit_daemon_completion_applies_first_valid_shutdown_signal() {
        let completion = evaluate_daemon_completion(10, &[8, 3, 7], Some(2), Some(4));
        assert_eq!(completion.executed_ticks, 5);
        assert_eq!(
            completion.completion_reason,
            "graceful-shutdown:signal@3;drain_ticks=2;timeout_ticks=4;ignored_signals=2"
        );
    }

    #[test]
    fn regression_daemon_completion_marks_late_shutdown_signals_as_ignored() {
        // Regression: #2674
        let completion = evaluate_daemon_completion(4, &[7, 8], Some(1), Some(1));
        assert_eq!(completion.executed_ticks, 4);
        assert_eq!(
            completion.completion_reason,
            "tick-budget-exhausted;ignored_signals=2"
        );
    }

    #[test]
    fn regression_daemon_completion_fails_closed_when_drain_exceeds_timeout() {
        // Regression: #2674
        let completion = evaluate_daemon_completion(9, &[7], Some(3), Some(1));
        assert_eq!(completion.executed_ticks, 8);
        assert_eq!(
            completion.completion_reason,
            "graceful-shutdown-timeout:signal@7;drain_ticks=3;timeout_ticks=1;ignored_signals=0"
        );
    }

    #[test]
    fn performance_daemon_completion_bounds_execution_by_timeout_deadline() {
        let completion = evaluate_daemon_completion(9, &[8], Some(50), Some(1));
        assert!(
            completion.executed_ticks <= 9,
            "timeout-bound completion must not exceed max tick budget"
        );
    }
}
