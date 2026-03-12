use super::{
    DataLayerM7BillingReconciliationDecision, DataLayerM7BillingReconciliationReport,
    DataLayerM7OwnerBillingDailyProjection, DataLayerM7TelemetryPointRecord,
    DataLayerM7TimeseriesError, DATA_LAYER_M7_AGGREGATE_REASON_CODE,
};
use kamn_data_layer::{
    project_data_layer_m7_owner_billing_daily,
    DataLayerM7BillingDailyProjection as ExtractedBillingProjection,
    DataLayerM7BillingProjectionSampleInput, DataLayerM7BillingReconciliationError,
    DataLayerM7BillingReconciliationReport as ExtractedBillingReport,
};

pub(crate) fn map_m7_billing_error_to_timeseries(
    error: DataLayerM7BillingReconciliationError,
) -> DataLayerM7TimeseriesError {
    match error {
        DataLayerM7BillingReconciliationError::InvalidBucketDayEpochSeconds(value) => {
            DataLayerM7TimeseriesError::InvalidBucketDayEpochSeconds(value)
        }
    }
}

pub(crate) fn project_m7_owner_billing_daily_rows(
    owner_did: &str,
    owner_points: &[DataLayerM7TelemetryPointRecord],
) -> Vec<DataLayerM7OwnerBillingDailyProjection> {
    project_m7_owner_billing_daily_projection(owner_did, owner_points)
        .into_iter()
        .map(core_m7_billing_daily_projection)
        .collect()
}

pub(crate) fn project_m7_owner_billing_daily_projection(
    owner_did: &str,
    owner_points: &[DataLayerM7TelemetryPointRecord],
) -> Vec<ExtractedBillingProjection> {
    project_data_layer_m7_owner_billing_daily(
        owner_did,
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
    )
}

pub(crate) fn core_m7_billing_daily_projection(
    projection: ExtractedBillingProjection,
) -> DataLayerM7OwnerBillingDailyProjection {
    DataLayerM7OwnerBillingDailyProjection {
        owner_did: projection.owner_did,
        bucket_day_epoch_seconds: projection.bucket_day_epoch_seconds,
        messages_stored_total: projection.messages_stored_total,
        bytes_stored_total: projection.bytes_stored_total,
        queries_executed_total: projection.queries_executed_total,
        embeddings_generated_total: projection.embeddings_generated_total,
        reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
    }
}

pub(crate) fn core_m7_billing_reconciliation_report(
    report: ExtractedBillingReport,
) -> DataLayerM7BillingReconciliationReport {
    DataLayerM7BillingReconciliationReport {
        owner_did: report.owner_did,
        bucket_day_epoch_seconds: report.bucket_day_epoch_seconds,
        decision: match report.decision {
            kamn_data_layer::DataLayerM7BillingReconciliationDecision::Match => {
                DataLayerM7BillingReconciliationDecision::Match
            }
            kamn_data_layer::DataLayerM7BillingReconciliationDecision::Mismatch => {
                DataLayerM7BillingReconciliationDecision::Mismatch
            }
        },
        reason_code: report.reason_code,
        projected_messages_stored_total: report.projected_messages_stored_total,
        projected_bytes_stored_total: report.projected_bytes_stored_total,
        projected_queries_executed_total: report.projected_queries_executed_total,
        projected_embeddings_generated_total: report.projected_embeddings_generated_total,
        statement_messages_stored_total: report.statement_messages_stored_total,
        statement_bytes_stored_total: report.statement_bytes_stored_total,
        statement_queries_executed_total: report.statement_queries_executed_total,
        statement_embeddings_generated_total: report.statement_embeddings_generated_total,
    }
}
