const QUOTA_POLICY_TEST_SURFACE: &str = include_str!("runtime_guard_quota_policy.rs");

#[test]
fn contract_runtime_guard_quota_policy_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_guard_quota_policy_allows_all_supported_scope_classes",
        "integration_runtime_guard_quota_policy_rejects_invalid_inputs_with_deterministic_reasons",
        "integration_runtime_guard_quota_policy_allows_limit_boundary_without_mutating_input",
        "integration_runtime_guard_quota_policy_reason_helpers_expose_deterministic_markers",
    ] {
        assert!(QUOTA_POLICY_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
