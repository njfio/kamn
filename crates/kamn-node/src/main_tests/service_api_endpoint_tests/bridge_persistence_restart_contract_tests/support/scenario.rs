use super::*;

pub(crate) enum LiveBridgeTestOverride {
    Success,
    Ambiguous,
}

pub(crate) struct LiveBridgeScenario {
    _env: ServiceApiTestEnvGuards,
    _transaction: LiveBridgeTransactionEnv,
    _override: crate::service_api_endpoint::TestLiveSolanaSettlementOverrideGuard,
    state_file: PathBuf,
    _state_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    bind_addr: String,
    caller_did: &'static str,
}

impl LiveBridgeScenario {
    pub(crate) fn new(
        label: &str,
        caller_did: &'static str,
        snapshot_addr: &str,
        override_kind: LiveBridgeTestOverride,
    ) -> Self {
        let env = acquire_service_api_test_env();
        let transaction = LiveBridgeTransactionEnv::enable(format!("{label}-keypair").as_str());
        let override_guard = override_guard(override_kind);
        let state_file = unique_named_state_file(label);
        let state_guard = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _transaction: transaction,
            _override: override_guard,
            state_file,
            _state_guard: state_guard,
            snapshot: build_bridge_snapshot(snapshot_addr),
            bind_addr: reserve_loopback_addr(),
            caller_did,
        }
    }

    pub(crate) fn submit(&self, nonce: u64, payload: &str) -> String {
        submit_bridge(
            &self.snapshot,
            self.bind_addr.as_str(),
            self.caller_did,
            nonce,
            payload,
        )["bridge_id"]
            .as_str()
            .expect("bridge id")
            .to_owned()
    }

    pub(crate) fn forward(&self, nonce: u64, bridge_id: &str) -> Value {
        forward_bridge(
            &self.snapshot,
            self.bind_addr.as_str(),
            self.caller_did,
            nonce,
            bridge_id,
        )
    }

    pub(crate) fn forward_response(&self, nonce: u64, bridge_id: &str) -> String {
        forward_bridge_response(
            &self.snapshot,
            self.bind_addr.as_str(),
            self.caller_did,
            nonce,
            bridge_id,
        )
    }

    pub(crate) fn state(&self) -> Value {
        read_state_json(self.state_file.as_path())
    }
}

impl Drop for LiveBridgeScenario {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.state_file);
    }
}

fn override_guard(
    kind: LiveBridgeTestOverride,
) -> crate::service_api_endpoint::TestLiveSolanaSettlementOverrideGuard {
    match kind {
        LiveBridgeTestOverride::Success => {
            crate::service_api_endpoint::set_test_live_solana_settlement_override(true)
        }
        LiveBridgeTestOverride::Ambiguous => {
            crate::service_api_endpoint::set_test_live_solana_settlement_ambiguous_after_submit()
        }
    }
}
