use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use kamn_e2e_harness::drivers::sdk_direct::SdkDirectDriver;
use kamn_e2e_harness::drivers::HarnessDriver;

#[test]
fn spec_c01_sdk_direct_live_toggle_disabled_fails_closed_without_probe_invocation() {
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_calls_for_closure = Arc::clone(&probe_calls);
    let driver = SdkDirectDriver::with_probe(false, move || {
        probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Err("probe should not run when live toggle is disabled".to_owned())
    });

    let result = driver.execute("S-01");
    assert_eq!(result.status, "fail");
    assert_eq!(
        probe_calls.load(Ordering::SeqCst),
        0,
        "probe must not be invoked when live toggle is disabled",
    );
}

#[test]
fn spec_c02_sdk_direct_live_s01_success_maps_to_pass() {
    let driver = SdkDirectDriver::with_probe(true, || Ok(()));
    let result = driver.execute("S-01");
    assert_eq!(result.status, "pass");
}

#[test]
fn spec_c03_sdk_direct_live_s01_failure_maps_to_fail() {
    let driver = SdkDirectDriver::with_probe(true, || Err("probe failure".to_owned()));
    let result = driver.execute("S-01");
    assert_eq!(result.status, "fail");
}

#[test]
fn spec_c04_sdk_direct_live_non_live_bound_scenario_remains_pass_without_probe_invocation() {
    let probe_calls = Arc::new(AtomicUsize::new(0));
    let probe_calls_for_closure = Arc::clone(&probe_calls);
    let driver = SdkDirectDriver::with_probe(true, move || {
        probe_calls_for_closure.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let result = driver.execute("S-99");
    assert_eq!(result.status, "pass");
    assert_eq!(
        probe_calls.load(Ordering::SeqCst),
        0,
        "non-live-bound scenario should not invoke live probe",
    );
}
