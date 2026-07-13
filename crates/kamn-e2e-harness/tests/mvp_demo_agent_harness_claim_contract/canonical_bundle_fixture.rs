use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use super::{live_binding_fixture, mvp_local_artifacts, three_agent::with_digest, write_file};

const SIGNATURE: &str = "5nSgnDevnetSignature111111111111111111111111111";
const PAYER: &str = "payer111111111111111111111111111111111111111";
const RECIPIENT: &str = "recipient11111111111111111111111111111111111";

pub(crate) fn write(root: &Path, report: String) -> PathBuf {
    mvp_local_artifacts::write_valid_local_artifacts(root);
    let binding = copy_binding(root);
    write_settlement_files(root);
    let bound = report.replace(
        &live_binding_fixture::binding_fixture()
            .path
            .display()
            .to_string(),
        &binding.display().to_string(),
    );
    let indexed = index_settlement_artifacts(root, &bound);
    write_report_pair(root, &indexed);
    write_report_pair(&root.parent().expect("run parent").join("latest"), &indexed);
    root.join("proof/report.json")
}

fn copy_binding(root: &Path) -> PathBuf {
    let source = live_binding_fixture::binding_fixture();
    let destination = root.join("proof/live-task-settlement-binding.json");
    write_file(
        &destination,
        std::fs::read_to_string(&source.path).expect("binding fixture should read"),
    );
    destination
}

fn write_settlement_files(root: &Path) {
    let response = confirmation_response();
    write_file(
        &root.join("proof/solana-confirmation-response.json"),
        response.clone(),
    );
    write_file(
        &root.join("proof/settlement-evidence.json"),
        evidence(&response),
    );
    write_file(
        &root.join("proof/devnet-settlement-output.txt"),
        settlement_log(),
    );
}

fn index_settlement_artifacts(root: &Path, report: &str) -> String {
    serde_json::from_str::<serde_json::Value>(report).expect("report fixture should parse");
    let evidence = root.join("proof/settlement-evidence.json");
    let confirmation = root.join("proof/solana-confirmation-response.json");
    let fields = format!(
        r#","devnet_settlement_evidence":"{}","solana_confirmation_response":"{}"}},"claim_matrix""#,
        evidence.display(),
        confirmation.display()
    );
    report.replacen(r#"},"claim_matrix""#, fields.as_str(), 1)
}

fn write_report_pair(root: &Path, report: &str) {
    let markdown =
        format!("# MVP demo proof\n\nhttps://explorer.solana.com/tx/{SIGNATURE}?cluster=devnet\n");
    write_file(&root.join("proof/report.json"), report.to_owned());
    write_file(&root.join("proof/report.md"), markdown);
}

fn evidence(response: &str) -> String {
    with_digest(
        format!(
            r#"{{"schema_version":"kamn.mvp.offline-settlement-evidence.v1","evidence_source":"solana-cli-confirm-and-balance-rpc","execution_surface":"command-override","network":"solana:devnet","rpc_url":"https://api.devnet.solana.com","payer_pubkey":"{PAYER}","recipient_pubkey":"{RECIPIENT}","lamports":1,"escrow_id":"escrow-three-agent-7045","task_id":"tx-three-agent-7045","task_binding_digest":"{}","settlement_tx_signature":"{SIGNATURE}","settlement_commitment":"finalized","payer_balance_before":20,"payer_balance_after":19,"recipient_balance_before":10,"recipient_balance_after":11,"persisted_settlement_tx_signature":"{SIGNATURE}","authoritative_rpc_digest":"sha256:{}","evidence_digest":""}}"#,
            live_binding_fixture::binding_fixture().digest,
            sha256(response)
        ),
        "evidence_digest",
    )
}

fn confirmation_response() -> String {
    format!(
        r#"{{"confirmationStatus":"finalized","meta":{{"err":null,"preBalances":[20,10],"postBalances":[19,11]}},"transaction":{{"signatures":["{SIGNATURE}"],"message":{{"accountKeys":["{PAYER}","{RECIPIENT}"]}}}}}}"#
    )
}

fn settlement_log() -> String {
    format!(
        "devnet_settlement_status=PASS\nnetwork=solana:devnet\nexecution_surface=command-override\nrpc_url=https://api.devnet.solana.com\npayer_pubkey={PAYER}\nrecipient_pubkey={RECIPIENT}\nlamports=1\nescrow_id=escrow-three-agent-7045\nsettlement_tx_signature={SIGNATURE}\nsettlement_commitment=finalized\npayer_balance_before=20\npayer_balance_after=19\nrecipient_balance_before=10\nrecipient_balance_after=11\npersisted_settlement_tx_signature={SIGNATURE}\ntask_id=tx-three-agent-7045\ntask_binding_digest={}\n",
        live_binding_fixture::binding_fixture().digest
    )
}

fn sha256(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
