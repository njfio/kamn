use super::super::*;
use super::context::{RuntimeEvidenceContext, RuntimeEvidenceIdentities, RuntimeEvidenceM6ToM11};
use super::m8_m11::{
    build_runtime_evidence_m10, build_runtime_evidence_m11, build_runtime_evidence_m8,
    build_runtime_evidence_m9,
};
use super::support::observability_health_label;

pub(super) fn build_runtime_evidence_m6_to_m11(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<RuntimeEvidenceM6ToM11, String> {
    let m6_projection_edge_count = build_runtime_evidence_m6(context, identities)?;
    let m7_observability_health = build_runtime_evidence_m7(context, identities)?;
    let m8_retention_due_count = build_runtime_evidence_m8(context, identities)?;
    let (m9_dispatch_ack_status, m9_dispatch_reason_code) =
        build_runtime_evidence_m9(context, identities)?;
    let m10_archived_partition_count = build_runtime_evidence_m10()?;
    let (m11_decision, m11_reason_codes_csv) = build_runtime_evidence_m11(context)?;
    Ok(RuntimeEvidenceM6ToM11 {
        m6_projection_edge_count,
        m7_observability_health,
        m8_retention_due_count,
        m9_dispatch_ack_status,
        m9_dispatch_reason_code,
        m10_archived_partition_count,
        m11_decision,
        m11_reason_codes_csv,
    })
}

fn build_runtime_evidence_m6(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<usize, String> {
    let mut m6_registry = DataLayerM6GraphRegistry::new();
    let (sender_node_id, recipient_node_id) = runtime_evidence_m6_node_ids(identities);
    register_runtime_evidence_m6_node_pair(
        &mut m6_registry,
        identities,
        &sender_node_id,
        &recipient_node_id,
    )?;
    register_runtime_evidence_m6_edge(
        &mut m6_registry,
        context,
        identities,
        sender_node_id,
        recipient_node_id,
    )?;
    let projection = m6_registry
        .export_portable_edge_projection(identities.owner_did)
        .map_err(|error| format!("m6 projection failed: {error}"))?;
    Ok(projection.len())
}

fn runtime_evidence_m6_node_ids(identities: &RuntimeEvidenceIdentities) -> (String, String) {
    (
        format!("node:{}", identities.sender_agent_did),
        format!("node:{}", identities.recipient_agent_did),
    )
}

fn register_runtime_evidence_m6_node_pair(
    registry: &mut DataLayerM6GraphRegistry,
    identities: &RuntimeEvidenceIdentities,
    sender_node_id: &str,
    recipient_node_id: &str,
) -> Result<(), String> {
    register_runtime_evidence_m6_node(
        registry,
        identities.owner_did,
        sender_node_id,
        identities.sender_agent_did.as_str(),
        "sender",
    )?;
    register_runtime_evidence_m6_node(
        registry,
        identities.owner_did,
        recipient_node_id,
        identities.recipient_agent_did.as_str(),
        "recipient",
    )
}

fn register_runtime_evidence_m6_edge(
    registry: &mut DataLayerM6GraphRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
    sender_node_id: String,
    recipient_node_id: String,
) -> Result<(), String> {
    registry
        .register_edge(DataLayerM6GraphEdgeInput {
            owner_did: identities.owner_did.to_owned(),
            edge_id: format!("edge:{}", context.message_id),
            relation: DataLayerM6GraphEdgeRelation::Messaged,
            from_node_id: sender_node_id,
            to_node_id: recipient_node_id,
            weight: 1.0,
            observed_at_epoch_seconds: context.event_epoch_seconds.saturating_add(5),
        })
        .map(|_| ())
        .map_err(|error| format!("m6 edge registration failed: {error}"))
}

fn register_runtime_evidence_m6_node(
    registry: &mut DataLayerM6GraphRegistry,
    owner_did: &str,
    node_id: &str,
    label: &str,
    actor_label: &str,
) -> Result<(), String> {
    registry
        .register_node(DataLayerM6GraphNodeInput {
            owner_did: owner_did.to_owned(),
            node_id: node_id.to_owned(),
            kind: DataLayerM6GraphNodeKind::Agent,
            label: label.to_owned(),
        })
        .map(|_| ())
        .map_err(|error| format!("m6 {actor_label} node registration failed: {error}"))
}

fn build_runtime_evidence_m7(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<String, String> {
    let mut m7_registry = DataLayerM7TelemetryRegistry::new();
    ingest_runtime_evidence_m7_point(&mut m7_registry, context, identities)?;
    let observability = project_runtime_evidence_m7_observability(&m7_registry, identities)?;
    Ok(observability_health_label(observability.snapshot.latest_health).to_owned())
}

fn ingest_runtime_evidence_m7_point(
    registry: &mut DataLayerM7TelemetryRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<(), String> {
    registry
        .ingest_point(DataLayerM7TelemetryPointInput {
            owner_did: identities.owner_did.to_owned(),
            agent_did: identities.sender_agent_did.clone(),
            timestamp_epoch_seconds: context.event_epoch_seconds.saturating_add(6),
            message_count: 1,
            bytes_stored: context.content_size_bytes as u64,
            query_count: 1,
            embedding_count: 1,
            embedding_anomaly_count: 0,
            ingress_latency_ms_p95: 120,
            egress_latency_ms_p95: 140,
            active_sessions: 1,
        })
        .map(|_| ())
        .map_err(|error| format!("m7 telemetry ingest failed: {error}"))
}

fn project_runtime_evidence_m7_observability(
    registry: &DataLayerM7TelemetryRegistry,
    identities: &RuntimeEvidenceIdentities,
) -> Result<kamn_core::DataLayerM7OwnerObservabilityReport, String> {
    registry
        .evaluate_owner_observability(
            DataLayerM7BillingQuery {
                requester_owner_did: identities.owner_did.to_owned(),
                owner_did: identities.owner_did.to_owned(),
            },
            ObservabilitySloProfile::baseline(),
        )
        .map_err(|error| format!("m7 observability projection failed: {error}"))
}
