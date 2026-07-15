use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const PAYER: &str = "2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T";
const RECIPIENT: &str = "FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe";
const SIGNATURE: &str = "devnet-signature-111";

pub(crate) struct PathGuard(String);

impl Drop for PathGuard {
    fn drop(&mut self) {
        std::env::set_var("PATH", self.0.as_str());
    }
}

pub(crate) fn install(root: &Path) -> PathGuard {
    write_state(root);
    write_executable(
        root.join("solana-keygen").as_path(),
        format!("#!/bin/sh\necho {PAYER}\n").as_str(),
    );
    write_executable(root.join("solana").as_path(), solana_script().as_str());
    let original = std::env::var("PATH").unwrap_or_default();
    std::env::set_var("PATH", format!("{}:{original}", root.display()));
    PathGuard(original)
}

pub(crate) fn state_source(root: &Path) -> PathBuf {
    root.join("persisted-service-state.json")
}

fn write_state(root: &Path) {
    let state = serde_json::json!({
        "schema_version": "kamn.runtime.service-api-message-store.v4",
        "authorization_grants": [{"action": "task:create"}],
        "tasks": {"task-local-bound-7086": task()},
        "escrows": {"escrow-local-bound-7086": escrow()},
        "settlement_intents": {"escrow-local-bound-7086": intent()},
    });
    std::fs::write(
        state_source(root),
        serde_json::to_vec(&state).expect("state JSON"),
    )
    .expect("persisted state");
}

fn task() -> serde_json::Value {
    serde_json::json!({"task_id":"task-local-bound-7086","state":"completed",
        "transaction_id":"transaction-live-7086","terms_digest":"a".repeat(64)})
}

fn escrow() -> serde_json::Value {
    serde_json::json!({"escrow_id":"escrow-local-bound-7086","state":"released",
        "task_id":"task-local-bound-7086","transaction_id":"transaction-live-7086",
        "amount_lamports":1000000,"network":"solana-devnet","terms_digest":"a".repeat(64),
        "settlement_receipt_hash":SIGNATURE,"settlement_tx_signature":SIGNATURE,
        "settlement_network":"solana:devnet","settlement_commitment":"finalized"})
}

fn intent() -> serde_json::Value {
    serde_json::json!({"settlement_intent_id":"intent-local-bound-7086",
        "escrow_id":"escrow-local-bound-7086","actor_did":"kamn:did:a",
        "idempotency_key":"release-local-bound-7086","recipient_pubkey":RECIPIENT,
        "amount_lamports":1000000,"network":"solana:devnet","expected_signature":SIGNATURE,
        "signed_transaction_digest":format!("sha256:{}","b".repeat(64)),
        "signed_transaction_json":"signed-transaction-secret","state":"confirmed",
        "submission_attempt_count":1})
}

fn solana_script() -> String {
    format!(
        r#"#!/bin/sh
cat <<'JSON'
{{"confirmationStatus":"finalized","meta":{{"err":null,"fee":5000,"preBalances":[2500000000,2500000000],"postBalances":[2498995000,2501000000]}},"transaction":{{"signatures":["{SIGNATURE}"],"message":{{"accountKeys":["{PAYER}","{RECIPIENT}"]}}}}}}
JSON
"#
    )
}

fn write_executable(path: &Path, body: &str) {
    std::fs::write(path, body).expect("fake executable");
    let mut permissions = std::fs::metadata(path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}
