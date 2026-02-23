use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use kamn_e2e_harness::drivers::cli_scripted::CliScriptedDriver;
use kamn_e2e_harness::drivers::HarnessDriver;

#[test]
fn spec_c01_cli_scripted_live_toggle_disabled_keeps_deterministic_pass_without_runner_invocation() {
    let runner_calls = Arc::new(AtomicUsize::new(0));
    let runner_calls_for_closure = Arc::clone(&runner_calls);
    let driver = CliScriptedDriver::with_runner(false, move || {
        runner_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Err("runner should not execute when live toggle is disabled".to_owned())
    });

    let result = driver.execute("S-01");
    assert_eq!(result.status, "pass");
    assert_eq!(
        runner_calls.load(Ordering::SeqCst),
        0,
        "runner must not be invoked when live toggle is disabled",
    );
}

#[test]
fn spec_c02_cli_scripted_live_s01_success_maps_to_pass() {
    let driver = CliScriptedDriver::with_runner(true, || Ok(()));
    let result = driver.execute("S-01");
    assert_eq!(result.status, "pass");
}

#[test]
fn spec_c03_cli_scripted_live_s01_failure_maps_to_fail() {
    let driver = CliScriptedDriver::with_runner(true, || Err("runner failure".to_owned()));
    let result = driver.execute("S-01");
    assert_eq!(result.status, "fail");
}

#[test]
fn spec_c04_cli_scripted_live_non_s01_remains_pass_without_runner_invocation() {
    let runner_calls = Arc::new(AtomicUsize::new(0));
    let runner_calls_for_closure = Arc::clone(&runner_calls);
    let driver = CliScriptedDriver::with_runner(true, move || {
        runner_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let result = driver.execute("S-09");
    assert_eq!(result.status, "pass");
    assert_eq!(
        runner_calls.load(Ordering::SeqCst),
        0,
        "non-live-bound scenarios should not invoke live runner",
    );
}
