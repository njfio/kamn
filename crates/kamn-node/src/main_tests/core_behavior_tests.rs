use super::{daemon_tests, runtime_tests};

#[test]
fn functional_kolme_live_strict_env_local_key_source_allows_with_local_override() {
    runtime_tests::functional_kolme_live_strict_env_local_key_source_allows_with_local_override();
}

#[test]
fn functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code() {
    runtime_tests::functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code();
}

#[test]
fn functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles() {
    runtime_tests::functional_runtime_kolme_live_continuous_mode_executes_multiple_cycles();
}

#[test]
fn integration_kolme_live_strict_managed_external_key_source_policy_passes() {
    runtime_tests::integration_kolme_live_strict_managed_external_key_source_policy_passes();
}

#[cfg(unix)]
#[test]
fn integration_runtime_daemon_applies_graceful_shutdown_on_os_signal() {
    daemon_tests::integration_runtime_daemon_applies_graceful_shutdown_on_os_signal();
}

#[test]
fn regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers() {
    daemon_tests::regression_runtime_daemon_shutdown_timeout_emits_structured_timeout_drain_markers(
    );
}
