use super::super::*;
use super::context::{RuntimeEvidenceContext, RuntimeEvidenceIdentities};
use super::support::{data_layer_m9_ack_status_label, m11_decision_label};

pub(super) fn build_runtime_evidence_m8(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<usize, String> {
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    register_runtime_evidence_m8_message(&mut m8_registry, context, identities)?;
    project_runtime_evidence_m8_retention_due(&m8_registry, context, identities)
}

fn register_runtime_evidence_m8_message(
    registry: &mut DataLayerM8ComplianceRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<(), String> {
    registry
        .register_message(DataLayerM8MessageRecordInput {
            owner_did: identities.owner_did.to_owned(),
            message_id: context.message_id.to_owned(),
            created_at_epoch_seconds: context.event_epoch_seconds,
            content_hash: format!("sha256:{}:content", context.message_id),
            hash_chain_prev: format!("sha256:{}:prev", context.message_id),
            retention_class: DataLayerM8RetentionClass::Standard,
            retention_extension_seconds: 0,
            wrapped_keys: vec![DataLayerM8WrappedCekInput {
                recipient_did: identities.recipient_agent_did.clone(),
                wrapped_cek: format!("wrapped:{}", context.message_id),
            }],
        })
        .map(|_| ())
        .map_err(|error| format!("m8 message registration failed: {error}"))
}

fn project_runtime_evidence_m8_retention_due(
    registry: &DataLayerM8ComplianceRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<usize, String> {
    let retention_due = registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: identities.owner_did.to_owned(),
                owner_did: identities.owner_did.to_owned(),
            },
            context.event_epoch_seconds.saturating_add(100_000_000),
        )
        .map_err(|error| format!("m8 retention projection failed: {error}"))?;
    Ok(retention_due.len())
}

pub(super) fn build_runtime_evidence_m9(
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<(String, String), String> {
    let mut m9_registry = DataLayerM9RealtimeDeliveryRegistry::new();
    connect_runtime_evidence_m9_presence(&mut m9_registry, context, identities)?;
    let outcome = dispatch_runtime_evidence_m9_message(&mut m9_registry, context, identities)?;
    Ok((
        data_layer_m9_ack_status_label(outcome.ack_status).to_owned(),
        outcome.reason_code.to_owned(),
    ))
}

fn connect_runtime_evidence_m9_presence(
    registry: &mut DataLayerM9RealtimeDeliveryRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<(), String> {
    registry
        .connect_presence(DataLayerM9PresenceConnectRequest {
            requester_owner_did: identities.owner_did.to_owned(),
            owner_did: identities.owner_did.to_owned(),
            agent_did: identities.recipient_agent_did.clone(),
            connected_since_epoch_seconds: context.event_epoch_seconds.saturating_add(7),
            last_heartbeat_epoch_seconds: context.event_epoch_seconds.saturating_add(7),
            gateway_node: "gateway-service-api-runtime".to_owned(),
            capabilities_active: vec!["ws".to_owned()],
        })
        .map(|_| ())
        .map_err(|error| format!("m9 presence connect failed: {error}"))
}

fn dispatch_runtime_evidence_m9_message(
    registry: &mut DataLayerM9RealtimeDeliveryRegistry,
    context: &RuntimeEvidenceContext<'_>,
    identities: &RuntimeEvidenceIdentities,
) -> Result<kamn_core::DataLayerM9DispatchOutcome, String> {
    registry
        .dispatch_message(DataLayerM9DispatchRequest {
            requester_owner_did: identities.owner_did.to_owned(),
            owner_did: identities.owner_did.to_owned(),
            sender_agent_did: identities.sender_agent_did.clone(),
            recipient_agent_did: identities.recipient_agent_did.clone(),
            message_id: context.message_id.to_owned(),
            dispatched_at_epoch_seconds: context.event_epoch_seconds.saturating_add(8),
        })
        .map_err(|error| format!("m9 dispatch failed: {error}"))
}

pub(super) fn build_runtime_evidence_m10() -> Result<usize, String> {
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(DataLayerM10PartitionRecordInput {
            partition_month_id: 202401,
            all_messages_shredded: true,
        })
        .map_err(|error| format!("m10 partition registration failed: {error}"))?;
    let archived = m10_registry
        .archive_due_partitions(DataLayerM10ArchiveDueRequest {
            now_month_id: 202602,
            active_retention_months: 2,
            object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        })
        .map_err(|error| format!("m10 archival projection failed: {error}"))?;
    Ok(archived.len())
}

pub(super) fn build_runtime_evidence_m11(
    context: &RuntimeEvidenceContext<'_>,
) -> Result<(String, String), String> {
    let closure = data_layer_m11_evaluate_closure_evidence(build_runtime_evidence_m11_input(context))
    .map_err(|error| format!("m11 closure evaluation failed: {error}"))?;
    Ok((
        m11_decision_label(closure.decision).to_owned(),
        closure.reason_codes.join(","),
    ))
}

fn build_runtime_evidence_m11_input(
    context: &RuntimeEvidenceContext<'_>,
) -> DataLayerM11ClosureEvidenceInput {
    DataLayerM11ClosureEvidenceInput {
        release_marker: format!("service-api-runtime:{}", context.message_id),
        hardening_report: build_runtime_evidence_m11_hardening_report(),
        critical_scenario_report: build_runtime_evidence_m11_critical_scenario_report(),
        performance_budget_met: true,
        security_signoff_complete: true,
        chaos_signoff_complete: true,
    }
}

fn build_runtime_evidence_m11_hardening_report() -> DataLayerM11OperatorReadinessReport {
    DataLayerM11OperatorReadinessReport {
        decision: DataLayerM11OperatorReadinessDecision::Go,
        reason_codes: vec!["m11_operator_readiness_go"],
        missing_required_scenario_ids: Vec::new(),
        failing_critical_scenario_ids: Vec::new(),
        total_required_scenarios: 1,
        passed_required_scenarios: 1,
    }
}

fn build_runtime_evidence_m11_critical_scenario_report(
) -> DataLayerPrdCriticalScenarioConformanceReport {
    DataLayerPrdCriticalScenarioConformanceReport {
        decision: DataLayerPrdCriticalScenarioConformanceDecision::Conformant,
        reason_codes: vec!["prd_critical_scenario_matrix_conformant"],
        missing_scenario_ids: Vec::new(),
        failed_scenario_ids: Vec::new(),
        shell_policy_violation_scenario_ids: Vec::new(),
        total_required_scenarios: 1,
        passed_required_scenarios: 1,
    }
}
