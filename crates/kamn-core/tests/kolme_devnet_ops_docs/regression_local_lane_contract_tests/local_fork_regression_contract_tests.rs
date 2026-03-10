use super::super::shared_support::{assert_plan_contains_all};

const REGRESSION_REQUIRES_LOCAL_FORK_SYNC_METADATA_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork metadata sync lane fails closed for checkout-path, remote-URL, ref, and dirty-checkout drift (`Regression: #1429`).",
];

#[test]
fn regression_requires_local_fork_sync_metadata_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_SYNC_METADATA_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_sync_metadata_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_SMOKE_EVIDENCE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork smoke evidence lane fails closed on missing local opt-in, metadata sync failure, command timeout, and smoke-command errors (`Regression: #1430`).",
];

#[test]
fn regression_requires_local_fork_smoke_evidence_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_SMOKE_EVIDENCE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_smoke_evidence_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_CHECKOUT_BOOTSTRAP_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork checkout bootstrap lane fails closed for local opt-in, checkout provenance drift, diagnostics command failures, and runtime budget overruns (`Regression: #1663`).",
];

#[test]
fn regression_requires_local_fork_checkout_bootstrap_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_CHECKOUT_BOOTSTRAP_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_checkout_bootstrap_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_MATRIX_PORTABLE_CARGO_PROFILE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork Rust test matrix portable cargo profile (`--cargo-profile portable`) remains fail-closed and linker-portable via `RUSTFLAGS=''` cargo override (`Regression: #1659`).",
];

#[test]
fn regression_requires_local_fork_matrix_portable_cargo_profile_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_MATRIX_PORTABLE_CARGO_PROFILE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_matrix_portable_cargo_profile_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_BOOTSTRAP_READINESS_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork bootstrap/readiness lane fails closed for sync/probe prerequisite failures, runtime budget overruns, and missing local opt-in (`Regression: #1488`).",
];

#[test]
fn regression_requires_local_fork_bootstrap_readiness_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_BOOTSTRAP_READINESS_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_bootstrap_readiness_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_PROCESS_LIFECYCLE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork process lifecycle integration lane fails closed for process start/readiness/integration/teardown/budget drift and missing local opt-in (`Regression: #1494`).",
];

#[test]
fn regression_requires_local_fork_process_lifecycle_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_PROCESS_LIFECYCLE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_process_lifecycle_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_PROFILE_PREFLIGHT_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork profile preflight lane fails closed for local opt-in, checkout/profile contract drift, probe command failures, and runtime budget overruns (`Regression: #1648`).",
];

#[test]
fn regression_requires_local_fork_profile_preflight_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_PROFILE_PREFLIGHT_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_profile_preflight_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_PROFILE_PREFLIGHT_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork profile preflight policy and contract-lane command/report drift remains fail-closed (`Regression: #1697`).",
];

#[test]
fn regression_requires_local_fork_profile_preflight_contract_lane_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_PROFILE_PREFLIGHT_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_profile_preflight_contract_lane_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_SELF_TEST_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork self-test lane fails closed for local opt-in, nested matrix/policy checkpoint failures, and runtime budget overruns (`Regression: #1652`).",
];

#[test]
fn regression_requires_local_fork_self_test_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_SELF_TEST_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_self_test_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_SELF_TEST_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork self-test policy and contract-lane command/report drift remains fail-closed (`Regression: #1702`).",
];

#[test]
fn regression_requires_local_fork_self_test_contract_lane_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_SELF_TEST_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_self_test_contract_lane_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_FORK_PORTABILITY_PREFLIGHT_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local fork portability preflight lane fails closed for local opt-in, mold linker drift, libudev dependency drift, and compile probe failures (`Regression: #1707`).",
];

#[test]
fn regression_requires_local_fork_portability_preflight_contract_lane_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_FORK_PORTABILITY_PREFLIGHT_CONTRACT_LANE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_fork_portability_preflight_contract_lane_guard_marker");
}
