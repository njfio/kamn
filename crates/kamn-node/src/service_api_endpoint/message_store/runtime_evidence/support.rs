use super::super::*;
use super::context::{RuntimeEvidenceM0ToM1, RuntimeEvidenceM2ToM5, RuntimeEvidenceM6ToM11};

pub(super) fn assemble_runtime_evidence_record(
    m0_to_m1: RuntimeEvidenceM0ToM1,
    m2_to_m5: RuntimeEvidenceM2ToM5,
    m6_to_m11: RuntimeEvidenceM6ToM11,
) -> ServiceApiDataLayerRuntimeEvidenceRecord {
    ServiceApiDataLayerRuntimeEvidenceRecord {
        schema_version: SERVICE_API_DATA_LAYER_RUNTIME_EVIDENCE_SCHEMA_VERSION.to_owned(),
        m0_content_hash: m0_to_m1.m0_content_hash,
        m1_merkle_root: m0_to_m1.m1_merkle_root,
        m2_authorization_reason_code: m2_to_m5.m2_authorization_reason_code,
        m2_audit_record_hash: m2_to_m5.m2_audit_record_hash,
        m3_blind_index_token: m2_to_m5.m3_blind_index_token,
        m3_match_count: m2_to_m5.m3_match_count,
        m4_transition_reason_code: m2_to_m5.m4_transition_reason_code,
        m5_record_hash: m2_to_m5.m5_record_hash,
        m6_projection_edge_count: m6_to_m11.m6_projection_edge_count,
        m7_observability_health: m6_to_m11.m7_observability_health,
        m8_retention_due_count: m6_to_m11.m8_retention_due_count,
        m9_dispatch_ack_status: m6_to_m11.m9_dispatch_ack_status,
        m9_dispatch_reason_code: m6_to_m11.m9_dispatch_reason_code,
        m10_archived_partition_count: m6_to_m11.m10_archived_partition_count,
        m11_decision: m6_to_m11.m11_decision,
        m11_reason_codes_csv: m6_to_m11.m11_reason_codes_csv,
    }
}

pub(super) fn m2_authorization_reason_code(decision: &DataLayerM2AuthorizationDecision) -> String {
    match decision {
        DataLayerM2AuthorizationDecision::Allow { reason_code }
        | DataLayerM2AuthorizationDecision::Deny { reason_code } => (*reason_code).to_owned(),
    }
}

pub(super) fn data_layer_m9_ack_status_label(status: DataLayerM9DispatchAckStatus) -> &'static str {
    match status {
        DataLayerM9DispatchAckStatus::Delivered => "delivered",
        DataLayerM9DispatchAckStatus::Queued => "queued",
    }
}

pub(super) fn m11_decision_label(decision: DataLayerM11ClosureAcceptanceDecision) -> &'static str {
    match decision {
        DataLayerM11ClosureAcceptanceDecision::Accepted => "accepted",
        DataLayerM11ClosureAcceptanceDecision::Rejected => "rejected",
    }
}

pub(super) fn observability_health_label(health: ObservabilityHealth) -> &'static str {
    match health {
        ObservabilityHealth::Healthy => "healthy",
        ObservabilityHealth::Degraded => "degraded",
        ObservabilityHealth::Critical => "critical",
    }
}
