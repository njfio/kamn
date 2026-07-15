use std::path::Path;

use super::devnet_settlement::{write_proof_file, DevnetSettlementEvidence};
use super::settlement_evidence_artifact::write_settlement_evidence_artifact;

pub(super) fn write_live_success_log(
    run_dir: &Path,
    evidence: &DevnetSettlementEvidence,
) -> Result<(), String> {
    write_settlement_evidence_artifact(run_dir, evidence)?;
    let mut content = format!(
        "devnet_settlement_status=PASS\nnetwork={}\nexecution_surface={}\nrpc_url={}\npayer_pubkey={}\nrecipient_pubkey={}\nlamports={}\nescrow_id={}\nsettlement_tx_signature={}\nsettlement_commitment={}\npayer_balance_before={}\npayer_balance_after={}\nrecipient_balance_before={}\nrecipient_balance_after={}\npersisted_settlement_tx_signature={}\ntask_id={}\ntask_binding_digest={}\n",
        evidence.network,
        evidence.execution_surface,
        evidence.rpc_url,
        evidence.payer_pubkey,
        evidence.recipient_pubkey,
        evidence.lamports,
        evidence.escrow_id,
        evidence.settlement_tx_signature,
        evidence.settlement_commitment,
        evidence.payer_balance_before,
        evidence.payer_balance_after,
        evidence.recipient_balance_before,
        evidence.recipient_balance_after,
        evidence.persisted_settlement_tx_signature,
        evidence.task_id.as_deref().unwrap_or("not-bound"),
        evidence.task_binding_digest.as_deref().unwrap_or("not-bound"),
    );
    content.push_str(provenance_log(evidence).as_str());
    write_proof_file(run_dir, "devnet-settlement-output.txt", content.as_str())
}

fn provenance_log(evidence: &DevnetSettlementEvidence) -> String {
    let values = (
        evidence.transaction_id.as_deref(),
        evidence.terms_digest.as_deref(),
        evidence.fee_lamports,
        evidence.settlement_receipt_hash.as_deref(),
        evidence.service_state_digest.as_deref(),
        evidence.settlement_intent_digest.as_deref(),
    );
    let (Some(transaction), Some(terms), Some(fee), Some(receipt), Some(state), Some(intent)) =
        values
    else {
        return String::new();
    };
    format!(
        "transaction_id={transaction}\nterms_digest={terms}\nfee_lamports={fee}\nsettlement_receipt_hash={receipt}\nservice_state_digest={state}\nsettlement_intent_digest={intent}\n"
    )
}
