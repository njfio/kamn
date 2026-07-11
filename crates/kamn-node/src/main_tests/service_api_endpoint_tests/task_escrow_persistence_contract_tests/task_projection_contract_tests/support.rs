use super::super::super::*;
use super::super::support::*;
use super::{CREATOR, OUTSIDER, PROVIDER, VERIFIER};

pub(super) struct ProjectionCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    state_file: std::path::PathBuf,
}

impl ProjectionCase {
    pub(super) fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file = unique_named_state_file(format!("kamn-projection-{label}").as_str());
        let (_, state_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_guard: state_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34250"),
            state_file,
        }
    }

    pub(super) fn seed_transaction(&self) -> String {
        let task_id = self.seed_task();
        let escrow = serde_json::json!({"task_id": task_id, "amount_lamports": 10_000});
        fund_escrow(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            3,
            escrow.to_string().as_str(),
        );
        task_id
    }

    pub(super) fn seed_task(&self) -> String {
        self.register_actors();
        let body = task_body();
        let task = create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            2,
            body.to_string().as_str(),
        );
        accept_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            PROVIDER,
            2,
            task.task_id.as_str(),
        );
        task.task_id
    }

    pub(super) fn query(&self, actor: &str, nonce: u64, task_id: &str, view: &str) -> String {
        let path = format!("/v1/tasks/{task_id}/{view}");
        raw_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method: "GET",
                path: path.as_str(),
                caller_did: actor,
                nonce,
                body: "",
                extra_headers: &[("X-KAMN-Authz-Scope", "tasks:read")],
            },
        )
    }

    pub(super) fn replace_escrow_transaction_id(&self, replacement: &str) {
        let mut state = read_state_json(self.state_file.as_path());
        let escrows = state["escrows"].as_object_mut().expect("escrow map");
        let escrow = escrows.values_mut().next().expect("bound escrow");
        escrow["transaction_id"] = serde_json::Value::String(replacement.to_owned());
        self.write_state(&state);
    }

    pub(super) fn remove_escrow_field(&self, field: &str) {
        let mut state = read_state_json(self.state_file.as_path());
        let escrows = state["escrows"].as_object_mut().expect("escrow map");
        let escrow = escrows.values_mut().next().expect("bound escrow");
        escrow.as_object_mut().expect("escrow record").remove(field);
        self.write_state(&state);
    }

    fn write_state(&self, state: &serde_json::Value) {
        fs::write(
            self.state_file.as_path(),
            serde_json::to_vec(state).expect("state json"),
        )
        .expect("write tampered state");
    }

    pub(super) fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }

    fn register_actors(&self) {
        for actor in [CREATOR, PROVIDER, VERIFIER, OUTSIDER] {
            register_agent_profile(
                &self.snapshot,
                reserve_loopback_addr().as_str(),
                actor,
                1,
                r#"{"agent_type":"agent","model_family":"pi","capabilities":["tasks"]}"#,
            );
        }
    }
}

fn task_body() -> serde_json::Value {
    serde_json::json!({
        "provider_did": test_service_api_sender_did(PROVIDER),
        "transaction_id": "transaction-projection-001",
        "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "idempotency_key": "create-projection-001",
        "description": "runtime disclosure projection",
    })
}
