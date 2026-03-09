use super::super::*;
use super::auth_fixture_support::test_service_api_auth_public_key_hex;

pub(crate) struct ServiceApiTestEnvGuards {
    _env_lock: MutexGuard<'static, ()>,
    _tls_mode_guard: EnvVarGuard,
    _tls_cert_guard: EnvVarGuard,
    _tls_key_guard: EnvVarGuard,
    _auth_public_key_guard: EnvVarGuard,
    _state_file_guard: EnvVarGuard,
    _log_level_guard: EnvVarGuard,
    _log_format_guard: EnvVarGuard,
    _chain_id_guard: EnvVarGuard,
    _sync_mode_guard: EnvVarGuard,
}

pub(crate) fn unique_service_api_test_state_file_path() -> String {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir()
        .join(format!(
            "kamn-node-service-api-state-test-{}-{entropy}.json",
            std::process::id()
        ))
        .to_string_lossy()
        .to_string()
}

pub(crate) fn acquire_service_api_test_env() -> ServiceApiTestEnvGuards {
    let state_file = unique_service_api_test_state_file_path();
    ServiceApiTestEnvGuards {
        _env_lock: lock_signer_env_guard(),
        _tls_mode_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", None),
        _tls_cert_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", None),
        _tls_key_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", None),
        _auth_public_key_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX",
            Some(test_service_api_auth_public_key_hex().as_str()),
        ),
        _state_file_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_STATE_FILE",
            Some(state_file.as_str()),
        ),
        _log_level_guard: EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", None),
        _log_format_guard: EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", None),
        _chain_id_guard: EnvVarGuard::set("KAMN_NODE_CHAIN_ID", None),
        _sync_mode_guard: EnvVarGuard::set("KAMN_NODE_SYNC_MODE", None),
    }
}
