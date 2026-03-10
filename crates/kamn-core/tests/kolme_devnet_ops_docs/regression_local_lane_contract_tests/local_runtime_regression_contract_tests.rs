use super::super::docs_assert_support::{assert_plan_contains_all};

const REGRESSION_REQUIRES_LOCAL_KOLME_API_PROBE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local Kolme API probe lane fails closed on unavailable health endpoint, invalid fork-info payload, and runtime budget overruns (`Regression: #1439`).",
];

#[test]
fn regression_requires_local_kolme_api_probe_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_KOLME_API_PROBE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_kolme_api_probe_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_KOLME_API_SMOKE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local Kolme API smoke lane fails closed without explicit local opt-in, probe prerequisite failure, smoke-command timeout, and smoke-command errors (`Regression: #1440`).",
];

#[test]
fn regression_requires_local_kolme_api_smoke_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_KOLME_API_SMOKE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_kolme_api_smoke_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_RUNTIME_COMMIT_LIVE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local runtime-commit live proof lane fails closed without local opt-in and for command timeout/failure paths (`Regression: #1450`).",
    "local runtime-commit live proof lane evidence policy remains fail-closed for missing live-provider command marker contracts (`Regression: #2095`).",
];

#[test]
fn regression_requires_local_runtime_commit_live_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_RUNTIME_COMMIT_LIVE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_runtime_commit_live_guard_marker");
}

const REGRESSION_REQUIRES_RUNTIME_COMMIT_BLOCK_FALLBACK_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "block fallback stale-window and response-height drift remains fail-closed (`Regression: #1464`).",
];

#[test]
fn regression_requires_runtime_commit_block_fallback_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_RUNTIME_COMMIT_BLOCK_FALLBACK_GUARD_MARKER_PLAN_MARKERS, "regression_requires_runtime_commit_block_fallback_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_NATIVE_API_PARITY_LIVE_PROOF_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local native API parity live proof lane fails closed without local opt-in and on nonce/broadcast/finality timeout or command failures (`Regression: #1465`).",
];

#[test]
fn regression_requires_local_native_api_parity_live_proof_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_NATIVE_API_PARITY_LIVE_PROOF_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_native_api_parity_live_proof_guard_marker");
}

const REGRESSION_REQUIRES_NATIVE_PARITY_DOCS_MATRIX_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "native parity fast/local command matrix docs drift remains fail-closed (`Regression: #1468`).",
];

#[test]
fn regression_requires_native_parity_docs_matrix_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_NATIVE_PARITY_DOCS_MATRIX_GUARD_MARKER_PLAN_MARKERS, "regression_requires_native_parity_docs_matrix_guard_marker");
}

const REGRESSION_REQUIRES_LIVE_KOLME_METHOD_AND_QUERY_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local probe fork-info query semantics and native parity broadcast method drift remain fail-closed (`Regression: #1482`).",
];

#[test]
fn regression_requires_live_kolme_method_and_query_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LIVE_KOLME_METHOD_AND_QUERY_GUARD_MARKER_PLAN_MARKERS, "regression_requires_live_kolme_method_and_query_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_LIVE_API_CONFORMANCE_HARNESS_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local live API conformance harness fails closed for probe/native parity prerequisite failures, runtime budget overruns, and endpoint contract drift (`Regression: #1483`).",
];

#[test]
fn regression_requires_local_live_api_conformance_harness_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_LIVE_API_CONFORMANCE_HARNESS_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_live_api_conformance_harness_guard_marker");
}

const REGRESSION_REQUIRES_LOCAL_KAMN_LIVE_RUNTIME_INTEGRATION_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local KAMN live runtime integration lane fails closed for bootstrap/localhost-signed/conformance/runtime-commit prerequisite drift, runtime budget overruns, and missing local opt-in (`Regression: #1489`).",
];

#[test]
fn regression_requires_local_kamn_live_runtime_integration_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCAL_KAMN_LIVE_RUNTIME_INTEGRATION_GUARD_MARKER_PLAN_MARKERS, "regression_requires_local_kamn_live_runtime_integration_guard_marker");
}

const REGRESSION_REQUIRES_LOCALHOST_SIGNED_RUNTIME_INTEGRATION_PREREQUISITE_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "local KAMN live runtime integration lane requires bounded localhost signed integration prerequisite execution before runtime commit submission (`Regression: #1636`).",
];

#[test]
fn regression_requires_localhost_signed_runtime_integration_prerequisite_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_LOCALHOST_SIGNED_RUNTIME_INTEGRATION_PREREQUISITE_GUARD_MARKER_PLAN_MARKERS, "regression_requires_localhost_signed_runtime_integration_prerequisite_guard_marker");
}

const REGRESSION_REQUIRES_UNIFIED_LOCAL_SIGNED_TO_KOLME_DEMO_GUARD_MARKER_PLAN_MARKERS: &[&str] = &[
    "unified local signed-to-Kolme demo lane fails closed for local opt-in, stage prerequisite drift, and runtime budget overruns (`Regression: #1640`).",
];

#[test]
fn regression_requires_unified_local_signed_to_kolme_demo_guard_marker() {
    assert_plan_contains_all(REGRESSION_REQUIRES_UNIFIED_LOCAL_SIGNED_TO_KOLME_DEMO_GUARD_MARKER_PLAN_MARKERS, "regression_requires_unified_local_signed_to_kolme_demo_guard_marker");
}
