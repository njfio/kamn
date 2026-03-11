use super::super::*;
use crate::service_api_endpoint::message_store::store::normalize_agent_did;

pub(super) struct RuntimeEvidenceContext<'a> {
    pub(super) message_id: &'a str,
    pub(super) payload: &'a str,
    pub(super) payload_tag: u64,
    pub(super) event_epoch_seconds: u64,
    pub(super) content_size_bytes: usize,
    pub(super) compressed_size_bytes: usize,
}

pub(super) struct RuntimeEvidenceIdentities {
    pub(super) sender_agent_did: String,
    pub(super) recipient_agent_did: String,
    pub(super) owner_did: &'static str,
    pub(super) owner_counterparty_did: &'static str,
}

pub(super) struct RuntimeEvidenceM0ToM1 {
    pub(super) m0_content_hash: String,
    pub(super) m1_merkle_root: String,
}

pub(super) struct RuntimeEvidenceM2ToM5 {
    pub(super) m2_authorization_reason_code: String,
    pub(super) m2_audit_record_hash: String,
    pub(super) m3_blind_index_token: String,
    pub(super) m3_match_count: usize,
    pub(super) m4_transition_reason_code: String,
    pub(super) m5_record_hash: String,
}

pub(super) struct RuntimeEvidenceM6ToM11 {
    pub(super) m6_projection_edge_count: usize,
    pub(super) m7_observability_health: String,
    pub(super) m8_retention_due_count: usize,
    pub(super) m9_dispatch_ack_status: String,
    pub(super) m9_dispatch_reason_code: String,
    pub(super) m10_archived_partition_count: usize,
    pub(super) m11_decision: String,
    pub(super) m11_reason_codes_csv: String,
}

pub(super) fn build_runtime_evidence_context<'a>(
    message_id: &'a str,
    payload: &'a str,
) -> RuntimeEvidenceContext<'a> {
    let payload_tag = deterministic_body_tag(payload.as_bytes());
    let content_size_bytes = payload.len().max(1);
    RuntimeEvidenceContext {
        message_id,
        payload,
        payload_tag,
        event_epoch_seconds: 1_708_560_000_u64.saturating_add(payload_tag % 10_000),
        content_size_bytes,
        compressed_size_bytes: (content_size_bytes / 2).max(1),
    }
}

pub(super) fn build_runtime_evidence_identities(
    sender_did: Option<&str>,
    recipient_did: Option<&str>,
) -> RuntimeEvidenceIdentities {
    let sender_agent_did =
        normalize_agent_did(sender_did, "kamn:did:agent:service-api-runtime-sender");
    let recipient_agent_did =
        build_runtime_evidence_recipient(sender_agent_did.as_str(), recipient_did);
    RuntimeEvidenceIdentities {
        sender_agent_did,
        recipient_agent_did,
        owner_did: "kamn:did:owner:service-api-runtime",
        owner_counterparty_did: "kamn:did:owner:service-api-runtime-recipient",
    }
}

fn build_runtime_evidence_recipient(sender_agent_did: &str, recipient_did: Option<&str>) -> String {
    let recipient_agent_did = normalize_agent_did(
        recipient_did,
        "kamn:did:agent:service-api-runtime-recipient",
    );
    if sender_agent_did == recipient_agent_did {
        "kamn:did:agent:service-api-runtime-recipient-alt".to_owned()
    } else {
        recipient_agent_did
    }
}
