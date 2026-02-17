use std::time::Duration;
#[cfg(test)]
use std::{cell::RefCell, thread::JoinHandle};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct DaemonCompletion {
    pub(super) executed_ticks: u64,
    pub(super) completion_reason: String,
}

#[cfg(test)]
fn os_signal_test_runtime_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum OsSignalTestKind {
    Sigint,
    Sigterm,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct OsSignalTestTrigger {
    delay_ms: u64,
    signal: OsSignalTestKind,
}

#[cfg(test)]
impl OsSignalTestTrigger {
    pub(super) const fn new(delay_ms: u64, signal: OsSignalTestKind) -> Self {
        Self { delay_ms, signal }
    }
}

#[cfg(test)]
thread_local! {
    static OS_SIGNAL_TEST_TRIGGERS: RefCell<Option<Vec<OsSignalTestTrigger>>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(super) fn configure_os_signal_test_triggers(triggers: Vec<OsSignalTestTrigger>) {
    OS_SIGNAL_TEST_TRIGGERS.with(|slot| {
        *slot.borrow_mut() = Some(triggers);
    });
}

#[cfg(test)]
fn take_os_signal_test_triggers() -> Vec<OsSignalTestTrigger> {
    OS_SIGNAL_TEST_TRIGGERS.with(|slot| slot.borrow_mut().take().unwrap_or_default())
}

#[cfg(all(test, unix))]
const TEST_SIGINT: i32 = 2;
#[cfg(all(test, unix))]
const TEST_SIGTERM: i32 = 15;

#[cfg(all(test, unix))]
unsafe extern "C" {
    fn raise(sig: i32) -> i32;
}

#[cfg(all(test, unix))]
fn raise_os_signal_for_test(signal: OsSignalTestKind) -> Result<(), String> {
    let signal_number = match signal {
        OsSignalTestKind::Sigint => TEST_SIGINT,
        OsSignalTestKind::Sigterm => TEST_SIGTERM,
    };
    let result = unsafe { raise(signal_number) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!("failed to raise test os signal: {signal:?}"))
    }
}

#[cfg(all(test, unix))]
fn spawn_os_signal_test_trigger_threads(
    triggers: Vec<OsSignalTestTrigger>,
) -> Vec<JoinHandle<Result<(), String>>> {
    triggers
        .into_iter()
        .map(|trigger| {
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(trigger.delay_ms));
                raise_os_signal_for_test(trigger.signal)
            })
        })
        .collect()
}

fn evaluate_daemon_completion_from_signal(
    max_ticks: u64,
    signal_tick: u64,
    ignored_signals: usize,
    drain_ticks: Option<u64>,
    timeout_ticks: Option<u64>,
) -> DaemonCompletion {
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

    evaluate_daemon_completion_from_signal(
        max_ticks,
        signal_tick,
        ignored_signals,
        drain_ticks,
        timeout_ticks,
    )
}

pub(super) fn evaluate_daemon_completion_with_os_signals(
    max_ticks: u64,
    tick_interval_ms: u64,
    drain_ticks: Option<u64>,
    timeout_ticks: Option<u64>,
) -> Result<DaemonCompletion, String> {
    #[cfg(unix)]
    {
        #[cfg(test)]
        let _signal_test_guard = os_signal_test_runtime_lock()
            .lock()
            .map_err(|_| "daemon os-signal test runtime lock poisoned".to_owned())?;

        let signal_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|error| format!("failed to build tokio signal runtime: {error}"))?;
        #[cfg(test)]
        let trigger_handles = spawn_os_signal_test_trigger_threads(take_os_signal_test_triggers());

        let runtime_result = signal_runtime.block_on(async move {
            let mut sigint_stream = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::interrupt(),
            )
            .map_err(|error| format!("failed to install SIGINT shutdown handler: {error}"))?;
            let mut sigterm_stream = tokio::signal::unix::signal(
                tokio::signal::unix::SignalKind::terminate(),
            )
            .map_err(|error| format!("failed to install SIGTERM shutdown handler: {error}"))?;

            let tick_duration = Duration::from_millis(tick_interval_ms);
            let mut first_signal_tick: Option<u64> = None;
            let mut ignored_signals = 0usize;
            for tick in 1..=max_ticks {
                if first_signal_tick.is_none() {
                    tokio::select! {
                        _ = sigint_stream.recv() => {
                            first_signal_tick = Some(tick);
                        }
                        _ = sigterm_stream.recv() => {
                            first_signal_tick = Some(tick);
                        }
                        _ = tokio::time::sleep(tick_duration) => {}
                    }
                } else {
                    tokio::select! {
                        _ = sigint_stream.recv() => {
                            ignored_signals = ignored_signals.saturating_add(1);
                        }
                        _ = sigterm_stream.recv() => {
                            ignored_signals = ignored_signals.saturating_add(1);
                        }
                        _ = tokio::time::sleep(tick_duration) => {}
                    }
                }

                if let Some(signal_tick) = first_signal_tick {
                    let completion = evaluate_daemon_completion_from_signal(
                        max_ticks,
                        signal_tick,
                        ignored_signals,
                        drain_ticks,
                        timeout_ticks,
                    );
                    if tick >= completion.executed_ticks {
                        return Ok(completion);
                    }
                }
            }

            if let Some(signal_tick) = first_signal_tick {
                return Ok(evaluate_daemon_completion_from_signal(
                    max_ticks,
                    signal_tick,
                    ignored_signals,
                    drain_ticks,
                    timeout_ticks,
                ));
            }
            Ok(DaemonCompletion {
                executed_ticks: max_ticks,
                completion_reason: "tick-budget-exhausted".to_owned(),
            })
        });

        #[cfg(test)]
        for handle in trigger_handles {
            let trigger_result = handle
                .join()
                .map_err(|_| "daemon os-signal test trigger thread panicked".to_owned())?;
            trigger_result?;
        }

        runtime_result
    }

    #[cfg(not(unix))]
    {
        let _ = (max_ticks, tick_interval_ms, drain_ticks, timeout_ticks);
        Err("daemon os signal shutdown is unsupported on this platform".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        configure_os_signal_test_triggers, evaluate_daemon_completion,
        evaluate_daemon_completion_with_os_signals, OsSignalTestKind, OsSignalTestTrigger,
    };
    #[cfg(unix)]
    use std::time::{Duration, Instant};

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
        configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(
            5,
            OsSignalTestKind::Sigterm,
        )]);
        let completion = evaluate_daemon_completion_with_os_signals(40, 1, Some(2), Some(5))
            .expect("daemon completion with OS signal handling should succeed");
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

    #[cfg(unix)]
    #[test]
    fn integration_daemon_completion_with_os_signals_applies_sigint_graceful_shutdown() {
        configure_os_signal_test_triggers(vec![OsSignalTestTrigger::new(
            5,
            OsSignalTestKind::Sigint,
        )]);
        let completion = evaluate_daemon_completion_with_os_signals(40, 1, Some(2), Some(5))
            .expect("daemon completion with SIGINT handling should succeed");
        assert!(
            completion
                .completion_reason
                .starts_with("graceful-shutdown:signal@"),
            "expected graceful shutdown completion reason for SIGINT, got {}",
            completion.completion_reason
        );
        assert!(
            completion.completion_reason.contains("ignored_signals=0"),
            "single SIGINT trigger should not increment ignored signals, got {}",
            completion.completion_reason
        );
        assert!(
            completion.executed_ticks <= 40,
            "SIGINT-driven shutdown should remain bounded by max ticks"
        );
    }

    #[cfg(unix)]
    #[test]
    fn regression_daemon_completion_with_os_signals_counts_replayed_signals() {
        // Regression: #3596
        configure_os_signal_test_triggers(vec![
            OsSignalTestTrigger::new(5, OsSignalTestKind::Sigterm),
            OsSignalTestTrigger::new(6, OsSignalTestKind::Sigint),
        ]);
        let completion = evaluate_daemon_completion_with_os_signals(60, 1, Some(5), Some(12))
            .expect("daemon completion with OS signal handling should succeed");
        assert!(
            completion.completion_reason.contains("ignored_signals=1"),
            "expected repeated signal count to be recorded, got {}",
            completion.completion_reason
        );
    }

    #[cfg(unix)]
    #[test]
    fn regression_daemon_completion_with_os_signals_without_signal_stays_bounded() {
        let start = Instant::now();
        let completion = evaluate_daemon_completion_with_os_signals(3, 1, Some(2), Some(3))
            .expect("daemon completion with os-signal handling should remain bounded");
        assert_eq!(completion.executed_ticks, 3);
        assert_eq!(completion.completion_reason, "tick-budget-exhausted");
        assert!(
            start.elapsed() < Duration::from_secs(1),
            "daemon os-signal no-signal path must remain bounded"
        );
    }

    #[cfg(unix)]
    #[test]
    fn regression_daemon_completion_with_os_signals_uses_tokio_signal_runtime_path() {
        // Regression: #2896
        let source = include_str!("daemon_shutdown.rs");
        assert!(
            source.contains("tokio::signal::unix::signal("),
            "expected daemon os signal path to use tokio unix signal streams"
        );
        assert!(
            source.contains("tokio::time::sleep("),
            "expected daemon os signal path to use tokio timer-driven tick cadence"
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
