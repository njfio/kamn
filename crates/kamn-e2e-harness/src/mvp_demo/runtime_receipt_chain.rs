use serde::Serialize;

use super::artifact_digest::attach_json_digest;
use super::pi_transaction_actor_model::{Actor, ServiceReceipt};
use super::pi_transaction_actor_verify::read_and_validate_actors;
use crate::service_receipt_commitment::{commitment, ReceiptProjection};

const SCHEMA: &str = "kamn.service.receipt-chain.v1";

#[derive(Serialize)]
struct Chain<'a> {
    schema_version: &'static str,
    task_id: &'a str,
    transaction_id: &'a str,
    escrow_id: &'a str,
    amount_lamports: u64,
    network: &'a str,
    settlement_tx_signature: &'a str,
    settlement_commitment: &'a str,
    receipt_chain_commitment: &'a str,
    service_receipt_commitment: &'a str,
    public_commitment: &'a str,
    actor_receipts: Vec<&'a ServiceReceipt>,
    chain_digest: &'static str,
}

/// Builds the portable service-authority summary from three verified v2 actor artifacts.
pub fn build_runtime_receipt_chain_from_actor_paths(paths: &[String; 3]) -> Result<String, String> {
    let actors = read_and_validate_actors(paths)?;
    let actor_receipts = ordered_receipts(&actors);
    let projection_commitment = receipt_commitment(&actor_receipts);
    let first = &actors[0];
    let raw = serde_json::to_string(&Chain {
        schema_version: SCHEMA,
        task_id: first.task_id.as_str(),
        transaction_id: first.transaction_id.as_str(),
        escrow_id: first.escrow_id.as_str(),
        amount_lamports: first.amount_lamports,
        network: first.network.as_str(),
        settlement_tx_signature: first.settlement_tx_signature.as_str(),
        settlement_commitment: first.settlement_commitment.as_str(),
        receipt_chain_commitment: first.receipt_chain_commitment.as_str(),
        service_receipt_commitment: projection_commitment.as_str(),
        public_commitment: first.public_commitment.as_str(),
        actor_receipts,
        chain_digest: "",
    })
    .map_err(|_| authority_error())?;
    Ok(attach_json_digest(raw, "chain_digest")?.json)
}

fn receipt_commitment(receipts: &[&ServiceReceipt]) -> String {
    let projected = receipts
        .iter()
        .map(|receipt| ReceiptProjection {
            actor_did: receipt.actor_did.as_str(),
            action: receipt.action.as_str(),
            resource_id: receipt.resource_id.as_str(),
            resulting_state: receipt.resulting_state.as_str(),
            receipt_id: receipt.service_receipt_id.as_str(),
            receipt_digest: receipt.service_receipt_digest.as_str(),
        })
        .collect::<Vec<_>>();
    commitment(&projected)
}

fn ordered_receipts(actors: &[Actor; 3]) -> Vec<&ServiceReceipt> {
    vec![
        &actors[0].service_receipts[0],
        &actors[1].service_receipts[0],
        &actors[0].service_receipts[1],
        &actors[1].service_receipts[1],
        &actors[0].service_receipts[2],
    ]
}

fn authority_error() -> String {
    "PI_SERVICE_AUTHORITY_MISMATCH".to_owned()
}
