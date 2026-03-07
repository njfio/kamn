use kamn_core::{
    DataLayerM7BillingQuery, DataLayerM7BillingReconciliationInput, DataLayerM7TelemetryPointInput,
    DataLayerM7TelemetryRegistry,
};
use kamn_data_layer::{
    project_data_layer_m7_owner_billing_daily, reconcile_data_layer_m7_owner_billing_daily,
    DataLayerM7BillingProjectionSampleInput, DataLayerM7BillingReconciliationInput as ExtractedInput,
};

fn telemetry_point(
    timestamp_epoch_seconds: u64,
    message_count: u64,
    bytes_stored: u64,
    query_count: u64,
    embedding_count: u64,
) -> DataLayerM7TelemetryPointInput {
    DataLayerM7TelemetryPointInput {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        agent_did: "kamn:did:agent:alpha-1".to_owned(),
        timestamp_epoch_seconds,
        message_count,
        bytes_stored,
        query_count,
        embedding_count,
        embedding_anomaly_count: 0,
        ingress_latency_ms_p95: 10,
        egress_latency_ms_p95: 15,
        active_sessions: 1,
    }
}

#[test]
fn integration_core_billing_wrappers_match_extracted_policy_outputs() {
    let mut registry = DataLayerM7TelemetryRegistry::new();
    registry
        .ingest_point(telemetry_point(1_708_560_100, 10, 2_000, 5, 3))
        .expect("first telemetry point should ingest");
    registry
        .ingest_point(telemetry_point(1_708_560_900, 2, 500, 1, 1))
        .expect("second telemetry point should ingest");

    let core_projection = registry
        .project_owner_billing_daily(DataLayerM7BillingQuery {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
        })
        .expect("core billing projection should succeed");
    let owner_points = registry
        .points_for_owner("kamn:did:owner:alpha")
        .expect("owner points should exist");
    let extracted_projection = project_data_layer_m7_owner_billing_daily(
        "kamn:did:owner:alpha",
        &owner_points
            .iter()
            .map(|point| DataLayerM7BillingProjectionSampleInput {
                bucket_day_epoch_seconds: point.bucket_day_epoch_seconds,
                message_count: point.message_count,
                bytes_stored: point.bytes_stored,
                query_count: point.query_count,
                embedding_count: point.embedding_count,
            })
            .collect::<Vec<_>>(),
    );
    assert_eq!(core_projection.len(), extracted_projection.len());
    assert_eq!(
        core_projection[0].messages_stored_total,
        extracted_projection[0].messages_stored_total
    );

    let day = core_projection.first().expect("projection row should exist");
    let core_reconciliation = registry
        .reconcile_owner_billing_daily(DataLayerM7BillingReconciliationInput {
            requester_owner_did: "kamn:did:owner:alpha".to_owned(),
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: day.bucket_day_epoch_seconds,
            messages_stored_total: day.messages_stored_total,
            bytes_stored_total: day.bytes_stored_total,
            queries_executed_total: day.queries_executed_total,
            embeddings_generated_total: day.embeddings_generated_total,
        })
        .expect("core reconciliation should succeed");
    let extracted_reconciliation = reconcile_data_layer_m7_owner_billing_daily(
        &extracted_projection,
        ExtractedInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: day.bucket_day_epoch_seconds,
            messages_stored_total: day.messages_stored_total,
            bytes_stored_total: day.bytes_stored_total,
            queries_executed_total: day.queries_executed_total,
            embeddings_generated_total: day.embeddings_generated_total,
        },
    )
    .expect("extracted reconciliation should succeed");
    assert_eq!(core_reconciliation.reason_code, extracted_reconciliation.reason_code);
    assert_eq!(
        core_reconciliation.projected_messages_stored_total,
        extracted_reconciliation.projected_messages_stored_total
    );
}
