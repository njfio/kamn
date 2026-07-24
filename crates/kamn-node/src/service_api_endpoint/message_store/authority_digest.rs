use super::{
    ServiceApiAuthorizationReceiptRecord, ServiceApiBridgeReceiptRecord,
    ServiceApiEscrowTransitionReceiptRecord, ServiceApiSettlementIntentRecord,
    ServiceApiTaskTransitionReceiptRecord,
};
use k256::sha2::{Digest, Sha256};

const PROFILE_DOMAIN: &str = "kamn.service.profile-authority.v1";
const TASK_DOMAIN: &str = "kamn.service.task-receipt.v1";
const ESCROW_DOMAIN: &str = "kamn.service.escrow-receipt.v1";
const AUTHORIZATION_DOMAIN: &str = "kamn.service.authorization-receipt.v1";
const SETTLEMENT_DOMAIN: &str = "kamn.service.settlement-intent.v1";
const BRIDGE_DOMAIN: &str = "kamn.service.bridge-receipt.v1";
const BRIDGE_PAYLOAD_DOMAIN: &str = "kamn.service.bridge-payload.v1";

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

pub(super) fn authorization(receipt: &ServiceApiAuthorizationReceiptRecord) -> String {
    digest(
        AUTHORIZATION_DOMAIN,
        &[
            receipt.receipt_id.as_str(),
            receipt.correlation_id.as_str(),
            receipt.actor_did.as_str(),
            receipt.resource.as_str(),
            receipt.action.as_str(),
            receipt.role.as_str(),
            receipt.decision.as_str(),
            receipt.reason_code.as_str(),
        ],
    )
}

pub(super) fn settlement(intent: &ServiceApiSettlementIntentRecord) -> String {
    let amount = intent.amount_lamports.to_string();
    digest(
        SETTLEMENT_DOMAIN,
        &[
            intent.settlement_intent_id.as_str(),
            intent.escrow_id.as_str(),
            intent.task_id.as_str(),
            intent.actor_did.as_str(),
            intent.idempotency_key.as_str(),
            intent.recipient_pubkey.as_str(),
            amount.as_str(),
            intent.asset.as_str(),
            intent.network.as_str(),
            intent.terms_digest.as_str(),
            intent.expected_signature.as_str(),
            intent.signed_transaction_digest.as_str(),
            intent.bridge_id.as_deref().unwrap_or(""),
            intent.bridge_receipt_id.as_deref().unwrap_or(""),
            intent.bridge_receipt_digest.as_deref().unwrap_or(""),
            intent.bridge_transaction_signature.as_deref().unwrap_or(""),
            intent.state.as_str(),
        ],
    )
}

pub(super) fn bridge(receipt: &ServiceApiBridgeReceiptRecord) -> String {
    let slot = receipt.finalized_slot.to_string();
    let amount = receipt
        .settlement_authority
        .as_ref()
        .map(|terms| terms.amount_lamports.to_string())
        .unwrap_or_default();
    let terms = receipt.settlement_authority.as_ref();
    digest(
        BRIDGE_DOMAIN,
        &[
            receipt.receipt_id.as_str(),
            receipt.bridge_id.as_str(),
            receipt.source_message_id.as_str(),
            receipt.target_network.as_str(),
            receipt.payload_hash.as_str(),
            terms.map(|value| value.escrow_id.as_str()).unwrap_or(""),
            terms.map(|value| value.task_id.as_str()).unwrap_or(""),
            terms.map(|value| value.actor_did.as_str()).unwrap_or(""),
            terms
                .map(|value| value.recipient_pubkey.as_str())
                .unwrap_or(""),
            amount.as_str(),
            terms.map(|value| value.asset.as_str()).unwrap_or(""),
            terms.map(|value| value.network.as_str()).unwrap_or(""),
            terms.map(|value| value.terms_digest.as_str()).unwrap_or(""),
            receipt.transaction_signature.as_str(),
            receipt.network.as_str(),
            receipt.commitment.as_str(),
            slot.as_str(),
            receipt.action.as_str(),
            receipt.resource_id.as_str(),
            receipt.state.as_str(),
        ],
    )
}

pub(super) fn bridge_payload(payload: &str) -> String {
    digest(BRIDGE_PAYLOAD_DOMAIN, &[payload])
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
