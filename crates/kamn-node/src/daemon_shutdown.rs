use std::thread;
use std::time::Duration;

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

pub(super) fn evaluate_daemon_completion_with_os_signals(
    max_ticks: u64,
    tick_interval_ms: u64,
    drain_ticks: Option<u64>,
    timeout_ticks: Option<u64>,
) -> Result<DaemonCompletion, String> {
    #[cfg(unix)]
    {
        os_signal::install_shutdown_handlers()?;
        let tick_duration = Duration::from_millis(tick_interval_ms);
        let mut first_signal_tick: Option<u64> = None;
        for tick in 1..=max_ticks {
            if first_signal_tick.is_none() && os_signal::shutdown_signal_observed() {
                first_signal_tick = Some(tick);
            }
            if let Some(signal_tick) = first_signal_tick {
                let completion = evaluate_daemon_completion(
                    max_ticks,
                    &[signal_tick],
                    drain_ticks,
                    timeout_ticks,
                );
                if tick >= completion.executed_ticks {
                    return Ok(completion);
                }
            }
            if tick < max_ticks {
                thread::sleep(tick_duration);
            }
        }
        if let Some(signal_tick) = first_signal_tick {
            return Ok(evaluate_daemon_completion(
                max_ticks,
                &[signal_tick],
                drain_ticks,
                timeout_ticks,
            ));
        }
        Ok(DaemonCompletion {
            executed_ticks: max_ticks,
            completion_reason: "tick-budget-exhausted".to_owned(),
        })
    }

    #[cfg(not(unix))]
    {
        let _ = (max_ticks, tick_interval_ms, drain_ticks, timeout_ticks);
        Err("daemon os signal shutdown is unsupported on this platform".to_owned())
    }
}

#[cfg(unix)]
mod os_signal {
    use std::sync::atomic::{AtomicBool, Ordering};

    static SHUTDOWN_SIGNAL_OBSERVED: AtomicBool = AtomicBool::new(false);

    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    const SIG_ERR: usize = usize::MAX;

    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }

    #[cfg(test)]
    unsafe extern "C" {
        fn raise(sig: i32) -> i32;
    }

    extern "C" fn shutdown_signal_handler(_signal: i32) {
        SHUTDOWN_SIGNAL_OBSERVED.store(true, Ordering::SeqCst);
    }

    pub(super) fn install_shutdown_handlers() -> Result<(), String> {
        SHUTDOWN_SIGNAL_OBSERVED.store(false, Ordering::SeqCst);
        let sigint_result = unsafe { signal(SIGINT, shutdown_signal_handler as usize) };
        if sigint_result == SIG_ERR {
            return Err("failed to install SIGINT shutdown handler".to_owned());
        }
        let sigterm_result = unsafe { signal(SIGTERM, shutdown_signal_handler as usize) };
        if sigterm_result == SIG_ERR {
            return Err("failed to install SIGTERM shutdown handler".to_owned());
        }
        Ok(())
    }

    pub(super) fn shutdown_signal_observed() -> bool {
        SHUTDOWN_SIGNAL_OBSERVED.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(super) fn raise_sigterm_for_test() -> Result<(), String> {
        let result = unsafe { raise(SIGTERM) };
        if result == 0 {
            Ok(())
        } else {
            Err("failed to raise SIGTERM".to_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use super::os_signal::raise_sigterm_for_test;
    use super::{evaluate_daemon_completion, evaluate_daemon_completion_with_os_signals};
    #[cfg(unix)]
    use std::thread;
    #[cfg(unix)]
    use std::time::Duration;

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

    #[cfg(unix)]
    #[test]
    fn integration_daemon_completion_with_os_signals_applies_graceful_shutdown() {
        let trigger = thread::spawn(|| {
            thread::sleep(Duration::from_millis(5));
            raise_sigterm_for_test().expect("SIGTERM test signal should be raised");
        });
        let completion = evaluate_daemon_completion_with_os_signals(40, 1, Some(2), Some(5))
            .expect("daemon completion with OS signal handling should succeed");
        trigger
            .join()
            .expect("signal trigger thread should complete");
        assert!(
            completion
                .completion_reason
                .starts_with("graceful-shutdown:signal@"),
            "expected graceful shutdown completion reason, got {}",
            completion.completion_reason
        );
        assert!(
            completion.executed_ticks <= 40,
            "signal-driven shutdown should remain bounded by max ticks"
        );
    }

    #[cfg(not(unix))]
    #[test]
    fn integration_daemon_completion_with_os_signals_is_unsupported_on_non_unix() {
        let result = evaluate_daemon_completion_with_os_signals(5, 1, Some(1), Some(1));
        assert!(
            result.is_err(),
            "non-unix targets should return unsupported error for OS signal handling"
        );
    }
}
