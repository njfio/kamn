const RETENTION_ENGINE_TEST_SURFACE: &str = include_str!("runtime_guard_retention_engine.rs");

#[test]
fn contract_runtime_guard_retention_engine_surface_exists_with_expected_markers() {
    for marker in [
        "integration_runtime_guard_retention_checker_rejects_invalid_inputs_and_allows_boundary",
        "integration_runtime_guard_retention_engine_status_uses_default_and_override_classes",
        "integration_runtime_guard_retention_engine_evaluate_returns_deterministic_expired_ids",
        "integration_runtime_guard_retention_engine_rejects_resurfaced_expired_record",
    ] {
        assert!(RETENTION_ENGINE_TEST_SURFACE.contains(marker), "missing marker: {marker}");
    }
}
