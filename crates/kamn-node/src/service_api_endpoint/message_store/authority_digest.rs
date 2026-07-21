use super::{ServiceApiEscrowTransitionReceiptRecord, ServiceApiTaskTransitionReceiptRecord};
use k256::sha2::{Digest, Sha256};

const PROFILE_DOMAIN: &str = "kamn.service.profile-authority.v1";
const TASK_DOMAIN: &str = "kamn.service.task-receipt.v1";
const ESCROW_DOMAIN: &str = "kamn.service.escrow-receipt.v1";

pub(super) fn profile(
    did: &str,
    reputation_score: u64,
    agent_type: &str,
    model_family: &str,
    capabilities: &[String],
) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, PROFILE_DOMAIN);
    append(&mut hasher, did);
    append(&mut hasher, reputation_score.to_string().as_str());
    append(&mut hasher, agent_type);
    append(&mut hasher, model_family);
    append(&mut hasher, capabilities.len().to_string().as_str());
    for capability in capabilities {
        append(&mut hasher, capability);
    }
    finish(hasher)
}

pub(super) fn task(receipt: &ServiceApiTaskTransitionReceiptRecord) -> String {
    let evidence_marker = if receipt.completion_evidence_digest.is_some() {
        "some"
    } else {
        "none"
    };
    digest(
        TASK_DOMAIN,
        &[
            receipt.receipt_id.as_str(),
            receipt.correlation_id.as_str(),
            receipt.idempotency_key.as_str(),
            receipt.actor_did.as_str(),
            receipt.task_id.as_str(),
            receipt.transaction_id.as_str(),
            receipt.action.as_str(),
            receipt.prior_state.as_str(),
            receipt.resulting_state.as_str(),
            receipt.terms_digest.as_str(),
            evidence_marker,
            receipt.completion_evidence_digest.as_deref().unwrap_or(""),
        ],
    )
}

pub(super) fn escrow(receipt: &ServiceApiEscrowTransitionReceiptRecord) -> String {
    let amount = receipt.amount_lamports.to_string();
    digest(
        ESCROW_DOMAIN,
        &[
            receipt.receipt_id.as_str(),
            receipt.correlation_id.as_str(),
            receipt.idempotency_key.as_str(),
            receipt.actor_did.as_str(),
            receipt.escrow_id.as_str(),
            receipt.task_id.as_str(),
            receipt.transaction_id.as_str(),
            receipt.action.as_str(),
            receipt.prior_state.as_str(),
            receipt.resulting_state.as_str(),
            receipt.network.as_str(),
            amount.as_str(),
            receipt.terms_digest.as_str(),
            receipt.release_policy.as_str(),
        ],
    )
}

fn digest(domain: &str, fields: &[&str]) -> String {
    let mut hasher = Sha256::new();
    append(&mut hasher, domain);
    for field in fields {
        append(&mut hasher, field);
    }
    finish(hasher)
}

fn append(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn finish(hasher: Sha256) -> String {
    format!("sha256:{}", hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
