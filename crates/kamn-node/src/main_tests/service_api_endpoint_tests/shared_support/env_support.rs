use super::super::*;
use super::auth_fixture_support::test_service_api_auth_public_key_hex;

pub(crate) struct ServiceApiTestEnvGuards {
    _env_lock: MutexGuard<'static, ()>,
    _tls_guards: ServiceApiTlsEnvGuards,
    _service_api_guards: ServiceApiCoreEnvGuards,
    _live_solana_guards: ServiceApiLiveSolanaEnvGuards,
    _node_guards: ServiceApiNodeEnvGuards,
}

struct ServiceApiTlsEnvGuards {
    _tls_mode_guard: EnvVarGuard,
    _tls_cert_guard: EnvVarGuard,
    _tls_key_guard: EnvVarGuard,
}

struct ServiceApiCoreEnvGuards {
    _auth_public_key_guard: EnvVarGuard,
    _state_file_guard: EnvVarGuard,
    _relay_spool_file_guard: EnvVarGuard,
    _audit_export_file_guard: EnvVarGuard,
}

struct ServiceApiLiveSolanaEnvGuards {
    _live_solana_bridge_rpc_guard: EnvVarGuard,
    _live_solana_settlement_keypair_guard: EnvVarGuard,
    _live_solana_settlement_recipient_guard: EnvVarGuard,
    _live_solana_settlement_lamports_guard: EnvVarGuard,
    _live_solana_settlement_commitment_guard: EnvVarGuard,
}

struct ServiceApiNodeEnvGuards {
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
        _tls_guards: service_api_tls_env_guards(),
        _service_api_guards: service_api_core_env_guards(state_file.as_str()),
        _live_solana_guards: service_api_live_solana_env_guards(),
        _node_guards: service_api_node_env_guards(),
    }
}

fn service_api_tls_env_guards() -> ServiceApiTlsEnvGuards {
    ServiceApiTlsEnvGuards {
        _tls_mode_guard: clear_env("KAMN_SERVICE_API_TLS_MODE"),
        _tls_cert_guard: clear_env("KAMN_SERVICE_API_TLS_CERT_FILE"),
        _tls_key_guard: clear_env("KAMN_SERVICE_API_TLS_KEY_FILE"),
    }
}

fn service_api_core_env_guards(state_file: &str) -> ServiceApiCoreEnvGuards {
    ServiceApiCoreEnvGuards {
        _auth_public_key_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX",
            Some(test_service_api_auth_public_key_hex().as_str()),
        ),
        _state_file_guard: EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file)),
        _relay_spool_file_guard: clear_env("KAMN_SERVICE_API_RELAY_SPOOL_FILE"),
        _audit_export_file_guard: clear_env("KAMN_SERVICE_API_AUDIT_EXPORT_FILE"),
    }
}

fn service_api_live_solana_env_guards() -> ServiceApiLiveSolanaEnvGuards {
    ServiceApiLiveSolanaEnvGuards {
        _live_solana_bridge_rpc_guard: clear_env("KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL"),
        _live_solana_settlement_keypair_guard: clear_env(
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE",
        ),
        _live_solana_settlement_recipient_guard: clear_env(
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
        ),
        _live_solana_settlement_lamports_guard: clear_env(
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS",
        ),
        _live_solana_settlement_commitment_guard: clear_env(
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT",
        ),
    }
}

fn service_api_node_env_guards() -> ServiceApiNodeEnvGuards {
    ServiceApiNodeEnvGuards {
        _log_level_guard: clear_env("KAMN_NODE_LOG_LEVEL"),
        _log_format_guard: clear_env("KAMN_NODE_LOG_FORMAT"),
        _chain_id_guard: clear_env("KAMN_NODE_CHAIN_ID"),
        _sync_mode_guard: clear_env("KAMN_NODE_SYNC_MODE"),
    }
}

fn clear_env(key: &'static str) -> EnvVarGuard {
    EnvVarGuard::set(key, None)
}
