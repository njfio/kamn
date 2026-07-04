use super::super::docs_assert_support::assert_plan_contains_all;

const PLAN_CONTAINS_LOCAL_FORK_SYNC_METADATA_LANE_PLAN_MARKERS: &[&str] = &[
    "## Deterministic Local Fork Sync Metadata Lane",
    "run_local_fork_sync_metadata_lane.sh",
    "kamn.kolme.local-fork-sync-metadata-summary.v1",
    "expected-remote-url https://github.com/njfio/kolme_fork.git",
];

#[test]
fn plan_contains_local_fork_sync_metadata_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_SYNC_METADATA_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_sync_metadata_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_SMOKE_EVIDENCE_LANE_PLAN_MARKERS: &[&str] = &[
    "## Bounded Local Fork Smoke Evidence Lane",
    "run_local_fork_smoke_evidence_lane.sh",
    "kamn.kolme.local-fork-smoke-evidence-summary.v1",
    "fork_smoke_command_timeout",
];

#[test]
fn plan_contains_local_fork_smoke_evidence_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_SMOKE_EVIDENCE_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_smoke_evidence_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_BOOTSTRAP_READINESS_CONTRACT_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Kolme Fork Bootstrap/Readiness Contract Lane",
    "run_local_kolme_fork_bootstrap_readiness_lane.sh",
    "check_local_kolme_fork_bootstrap_readiness_policy.py",
    "run_local_kolme_fork_bootstrap_readiness_contract_lane.sh",
    "kamn.kolme.local-fork-bootstrap-readiness-summary.v1",
];

#[test]
fn plan_contains_local_fork_bootstrap_readiness_contract_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_BOOTSTRAP_READINESS_CONTRACT_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_bootstrap_readiness_contract_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_PROCESS_LIFECYCLE_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Kolme Fork Process Lifecycle Integration Lane",
    "run_local_kolme_fork_process_lifecycle_lane.sh",
    "check_local_kolme_fork_process_lifecycle_policy.py",
    "run_local_kolme_fork_process_lifecycle_contract_lane.sh",
    "kamn.kolme.local-fork-process-lifecycle-summary.v1",
];

#[test]
fn plan_contains_local_fork_process_lifecycle_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_PROCESS_LIFECYCLE_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_process_lifecycle_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_PROFILE_PREFLIGHT_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Fork Profile Preflight Lane",
    "run_local_kolme_fork_profile_preflight_lane.sh",
    "check_local_kolme_fork_profile_preflight_policy.py",
    "run_local_kolme_fork_profile_preflight_contract_lane.sh",
    "kamn.kolme.local-fork-profile-preflight-summary.v1",
    "kamn.kolme.local-fork-profile-preflight-policy-report.v1",
];

#[test]
fn plan_contains_local_fork_profile_preflight_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_PROFILE_PREFLIGHT_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_profile_preflight_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_SELF_TEST_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Fork Self-Test Lane",
    "run_local_kolme_fork_self_test_lane.sh",
    "--matrix-cargo-profile portable",
    "check_local_kolme_fork_self_test_policy.py",
    "run_local_kolme_fork_self_test_contract_lane.sh",
    "kamn.kolme.local-fork-self-test-summary.v1",
    "kamn.kolme.local-fork-self-test-policy-report.v1",
];

#[test]
fn plan_contains_local_fork_self_test_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_SELF_TEST_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_self_test_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_PORTABILITY_PREFLIGHT_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Fork Portability Preflight Lane",
    "run_local_kolme_fork_portability_preflight_lane.sh",
    "check_local_kolme_fork_portability_preflight_policy.py",
    "run_local_kolme_fork_portability_preflight_contract_lane.sh",
    "kamn.kolme.local-fork-portability-preflight-summary.v1",
    "kamn.kolme.local-fork-portability-preflight-policy-report.v1",
];

#[test]
fn plan_contains_local_fork_portability_preflight_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_PORTABILITY_PREFLIGHT_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_portability_preflight_lane",
    );
}

const PLAN_CONTAINS_LOCAL_FORK_CHECKOUT_BOOTSTRAP_LANE_PLAN_MARKERS: &[&str] = &[
    "## Local Fork Checkout Bootstrap Lane",
    "run_local_kolme_fork_checkout_bootstrap_lane.sh",
    "check_local_kolme_fork_checkout_bootstrap_policy.py",
    "run_local_kolme_fork_checkout_bootstrap_contract_lane.sh",
    "kamn.kolme.local-fork-checkout-bootstrap-summary.v1",
];

#[test]
fn plan_contains_local_fork_checkout_bootstrap_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_LOCAL_FORK_CHECKOUT_BOOTSTRAP_LANE_PLAN_MARKERS,
        "plan_contains_local_fork_checkout_bootstrap_lane",
    );
}

const PLAN_CONTAINS_REAL_FORK_LOCAL_PROCESS_WRAPPER_LANE_PLAN_MARKERS: &[&str] = &[
    "## Real Fork Local Process Wrapper Contract Lane",
    "run_local_kolme_fork_real_process_contract_lane.sh",
    "run_local_kolme_fork_checkout_bootstrap_lane.sh",
    "check_local_kolme_fork_checkout_bootstrap_policy.py",
    "run_local_kolme_fork_profile_preflight_lane.sh",
    "check_local_kolme_fork_profile_preflight_policy.py",
    "run_local_kolme_fork_self_test_lane.sh",
    "check_local_kolme_fork_self_test_policy.py",
    "check_local_kolme_fork_real_process_policy.py",
    "kamn.kolme.local-fork-real-process-summary.v1",
];

#[test]
fn plan_contains_real_fork_local_process_wrapper_lane() {
    assert_plan_contains_all(
        PLAN_CONTAINS_REAL_FORK_LOCAL_PROCESS_WRAPPER_LANE_PLAN_MARKERS,
        "plan_contains_real_fork_local_process_wrapper_lane",
    );
}

const PLAN_CONTAINS_REAL_FORK_WRAPPER_POLICY_CHECKER_TEST_COMMAND_PLAN_MARKERS: &[&str] =
    &["test_check_local_kolme_fork_real_process_policy.sh"];

#[test]
fn plan_contains_real_fork_wrapper_policy_checker_test_command() {
    assert_plan_contains_all(
        PLAN_CONTAINS_REAL_FORK_WRAPPER_POLICY_CHECKER_TEST_COMMAND_PLAN_MARKERS,
        "plan_contains_real_fork_wrapper_policy_checker_test_command",
    );
}
