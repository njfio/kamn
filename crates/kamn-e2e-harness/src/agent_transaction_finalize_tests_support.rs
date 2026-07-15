use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::agent_transaction_evidence::AgentTransactionEvidencePaths;
use crate::{parse_agent_transaction_demo_config, AgentTransactionDemoConfig};

#[path = "../tests/support/pi_transaction_actor_fixture.rs"]
mod actor_fixture;
use actor_fixture::{ActorFixture, Overrides};

#[path = "agent_transaction_finalize_tests_fake_solana.rs"]
mod fake_solana;
use fake_solana::install_fake_solana;

#[path = "agent_transaction_finalize_tests_live_evidence.rs"]
mod live_evidence;
use live_evidence::write_live_evidence;

const RECIPIENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY";
const LAMPORTS_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS";
const COMMITMENT_ENV: &str = "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT";
const RECIPIENT: &str = "FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe";

pub(super) struct ProofRetryFixture {
    root: PathBuf,
    original_path: String,
    pub(super) config: AgentTransactionDemoConfig,
    pub(super) paths: AgentTransactionEvidencePaths,
}

impl ProofRetryFixture {
    pub(super) fn new() -> Self {
        let root = unique_root();
        let actors = ActorFixture::new();
        actors.write_all(Overrides::default());
        actors.rebind_shared_facts();
        let paths = evidence_paths(actors.paths(), write_live_evidence(&root));
        let config = config(&root);
        write_persisted_state(&root);
        let original_path = install_fake_solana(&root);
        std::fs::create_dir_all(&config.output_root).expect("output root");
        Self {
            root,
            original_path,
            config,
            paths,
        }
    }

    pub(super) fn block_latest_publication(&self) {
        std::fs::write(self.root.join("demo/latest"), "publication blocker")
            .expect("latest blocker");
    }

    pub(super) fn unblock_latest_publication(&self) {
        std::fs::remove_file(self.root.join("demo/latest")).expect("remove latest blocker");
    }

    pub(super) fn solana_calls(&self) -> String {
        std::fs::read_to_string(self.root.join("solana-calls.log")).expect("Solana call log")
    }

    pub(super) fn report(&self) -> String {
        std::fs::read_to_string(self.root.join("demo/latest/proof/report.json"))
            .expect("latest report")
    }

    pub(super) fn tamper_persisted_recipient(&self) {
        let path = self.root.join("staging/service-api-state.json");
        let raw = std::fs::read_to_string(&path).expect("persisted state");
        std::fs::write(path, raw.replace(RECIPIENT, "foreign-recipient"))
            .expect("tampered persisted state");
    }
}

impl Drop for ProofRetryFixture {
    fn drop(&mut self) {
        std::env::set_var("PATH", self.original_path.as_str());
    }
}

fn config(root: &Path) -> AgentTransactionDemoConfig {
    let mut config = parse_agent_transaction_demo_config(&config_env(root)).expect("config");
    config.output_root = root.join("demo").display().to_string();
    config.staging_root = root.join("staging").display().to_string();
    config.localhost_signed_demo_command = Some(localhost_command());
    config.service_api_vertical_slice_command = Some(service_command(
        "integration_service_api_endpoint_working_vertical_slice_proves_delivery_dispatch_and_audit_evidence",
    ));
    config.service_api_websocket_command = Some(service_command(
        "integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event",
    ));
    config
}

fn config_env(root: &Path) -> BTreeMap<String, String> {
    let mut values = static_config_env();
    values.insert(
        "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE".to_owned(),
        root.join("payer.json").display().to_string(),
    );
    values
}

fn static_config_env() -> BTreeMap<String, String> {
    let mut values = core_config_env();
    for (key, value) in [
        ("KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE", "a"),
        ("KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE", "b"),
        ("KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE", "c"),
    ] {
        values.insert(key.to_owned(), value.to_owned());
    }
    values
}

fn core_config_env() -> BTreeMap<String, String> {
    let values = [
        ("KAMN_MVP_AGENT_DRIVER", "pi".to_owned()),
        ("KAMN_MVP_DEVNET_MODE", "required".to_owned()),
        (
            "KAMN_MVP_SOLANA_RPC_URL",
            "https://api.devnet.solana.com".to_owned(),
        ),
        (RECIPIENT_ENV, RECIPIENT.to_owned()),
        (LAMPORTS_ENV, "1000000".to_owned()),
        (COMMITMENT_ENV, "finalized".to_owned()),
    ];
    values
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

fn evidence_paths(actors: [String; 3], live: [String; 4]) -> AgentTransactionEvidencePaths {
    AgentTransactionEvidencePaths {
        handoff: live[0].clone(),
        agent_a_receipt: live[1].clone(),
        agent_b_receipt: live[2].clone(),
        agent_c_observation: live[3].clone(),
        actors,
        run_id: "proof-retry".to_owned(),
    }
}

fn write_persisted_state(root: &Path) {
    let path = root.join("staging/service-api-state.json");
    let state = serde_json::json!({
        "schema_version": "kamn.runtime.service-api-message-store.v4",
        "tasks": {"task-local-bound-7086": {
            "task_id": "task-local-bound-7086", "state": "completed",
            "transaction_id": "transaction-live-7086", "terms_digest": "a".repeat(64),
        }},
        "escrows": {"escrow-local-bound-7086": {
            "escrow_id": "escrow-local-bound-7086", "state": "released",
            "task_id": "task-local-bound-7086", "transaction_id": "transaction-live-7086",
            "amount_lamports": 1000000, "network": "solana-devnet",
            "terms_digest": "a".repeat(64), "settlement_receipt_hash": "devnet-signature-111",
            "settlement_tx_signature": "devnet-signature-111",
            "settlement_network": "solana:devnet", "settlement_commitment": "finalized",
        }},
        "settlement_intents": {"escrow-local-bound-7086": {
            "settlement_intent_id": "intent-local-bound-7086", "escrow_id": "escrow-local-bound-7086",
            "actor_did": "kamn:did:a", "idempotency_key": "release-local-bound-7086",
            "recipient_pubkey": RECIPIENT, "amount_lamports": 1000000,
            "network": "solana:devnet", "expected_signature": "devnet-signature-111",
            "signed_transaction_digest": format!("sha256:{}", "b".repeat(64)),
            "signed_transaction_json": "signed-transaction-secret", "state": "confirmed",
            "submission_attempt_count": 1,
        }},
    });
    std::fs::write(path, serde_json::to_vec(&state).expect("state JSON"))
        .expect("persisted service state");
}

fn localhost_command() -> Vec<String> {
    vec!["sh".to_owned(), "-c".to_owned(), r#"cat > "$2" <<'JSON'
{"schema_version":"kamn.sdk.localhost-signed.demo-receipt-artifact.v1","status": "pass","signed_exchange":{"from":"kamn:did:agent:alice","to":"kamn:did:agent:bob","verified": true},"signed_flow":"task"}
JSON
echo receipt_reconciliation=GO
echo 'localhost signed message demo completed.'"#.to_owned(), "stub".to_owned()]
}

fn service_command(test_name: &str) -> Vec<String> {
    vec![
        "sh".to_owned(),
        "-c".to_owned(),
        format!("echo 'test {test_name} ... ok'; echo 'test result: ok'"),
    ]
}

fn unique_root() -> PathBuf {
    let id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-proof-retry-{}-{id}", std::process::id()))
}
