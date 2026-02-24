#[test]
fn regression_run_path_service_api_runtime_mode_classifier_rejects_non_api_non_full_modes() {
    for runtime_mode in ["bootstrap", "planning", "recovery-check", "daemon", "kolme-live"] {
        let error = classify_service_api_endpoint_runtime_path(runtime_mode).expect_err(
            "service-api runtime-mode classifier must fail closed for non-api/non-full modes",
        );
        assert!(
            matches!(
                error,
                ConfigError::RuntimeDaemonLifecycle(message)
                    if message.contains("service api endpoint requires runtime-mode api or full")
            ),
            "runtime mode {runtime_mode} must emit deterministic fail-closed reason text"
        );
    }
}

#[test]
fn regression_run_path_service_api_runtime_mode_classifier_routes_api_and_full_modes_deterministically(
) {
    assert_eq!(
        classify_service_api_endpoint_runtime_path("api")
            .expect("api runtime mode should classify"),
        ServiceApiEndpointRuntimePath::ServeInProcess
    );
    assert_eq!(
        classify_service_api_endpoint_runtime_path("full")
            .expect("full runtime mode should classify"),
        ServiceApiEndpointRuntimePath::SkipForFullSupervisor
    );
}

#[test]
fn regression_run_path_observability_full_supervisor_skip_gate_is_explicit() {
    assert!(should_skip_observability_endpoint_for_full_supervisor(
        "full"
    ));
    for runtime_mode in ["bootstrap", "planning", "recovery-check", "daemon", "api", "kolme-live"]
    {
        assert!(
            !should_skip_observability_endpoint_for_full_supervisor(runtime_mode),
            "runtime mode {runtime_mode} must not skip observability endpoint in-process path"
        );
    }
}
