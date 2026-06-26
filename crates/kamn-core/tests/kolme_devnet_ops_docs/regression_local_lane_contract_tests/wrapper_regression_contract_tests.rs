use super::super::docs_assert_support::assert_plan_contains_all;

const REGRESSION_REQUIRES_REAL_FORK_LOCAL_PROCESS_WRAPPER_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "real-fork local process wrapper lane fails closed for local opt-in, serve-command profile drift, self-test/lifecycle/policy checkpoint failure, and runtime budget overruns (`Regression: #1644`).",
];

#[test]
fn regression_requires_real_fork_local_process_wrapper_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_REAL_FORK_LOCAL_PROCESS_WRAPPER_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_real_fork_local_process_wrapper_guard_marker",
    );
}

const REGRESSION_REQUIRES_REAL_FORK_WRAPPER_BOOTSTRAP_PREREQUISITE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "real-fork local process wrapper bootstrap-first prerequisite ordering remains fail-closed for bootstrap lane/policy checkpoint drift (`Regression: #1667`).",
];

#[test]
fn regression_requires_real_fork_wrapper_bootstrap_prerequisite_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_REAL_FORK_WRAPPER_BOOTSTRAP_PREREQUISITE_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_real_fork_wrapper_bootstrap_prerequisite_guard_marker",
    );
}

const REGRESSION_REQUIRES_REAL_FORK_WRAPPER_POLICY_CHECKER_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "real-fork local process wrapper policy checker lane remains fail-closed for schema/contracts/checkpoint drift (`Regression: #1671`).",
];

#[test]
fn regression_requires_real_fork_wrapper_policy_checker_guard_marker() {
    assert_plan_contains_all(
        REGRESSION_REQUIRES_REAL_FORK_WRAPPER_POLICY_CHECKER_GUARD_MARKER_PLAN_MARKERS,
        "regression_requires_real_fork_wrapper_policy_checker_guard_marker",
    );
}
