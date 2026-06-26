use super::{
    authorize_owner_scope, core_m7_billing_reconciliation_report,
    data_layer_m7_project_observability_sample, map_m7_billing_error_to_timeseries,
    map_observability_error_to_timeseries, project_m7_owner_billing_daily_projection,
    project_m7_owner_billing_daily_rows, validate_daily_bucket, DataLayerM7BillingQuery,
    DataLayerM7BillingReconciliationInput, DataLayerM7BillingReconciliationReport,
    DataLayerM7OwnerBillingDailyProjection, DataLayerM7OwnerObservabilityReport,
    DataLayerM7TelemetryPointRecord, DataLayerM7TimeseriesError,
    DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
};
use crate::{ObservabilityMonitor, ObservabilitySloProfile};
use kamn_data_layer::{
    reconcile_data_layer_m7_owner_billing_daily,
    DataLayerM7BillingReconciliationInput as ExtractedBillingInput,
};

pub(crate) fn project_owner_billing_daily(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    query: DataLayerM7BillingQuery,
) -> Result<Vec<DataLayerM7OwnerBillingDailyProjection>, DataLayerM7TimeseriesError> {
    authorize_owner_scope(
        query.requester_owner_did.as_str(),
        query.owner_did.as_str(),
        DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
    )?;
    Ok(project_m7_owner_billing_daily_rows(
        query.owner_did.as_str(),
        owner_points,
    ))
}

pub(crate) fn reconcile_owner_billing_daily(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    input: DataLayerM7BillingReconciliationInput,
) -> Result<DataLayerM7BillingReconciliationReport, DataLayerM7TimeseriesError> {
    authorize_owner_scope(
        input.requester_owner_did.as_str(),
        input.owner_did.as_str(),
        DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
    )?;
    validate_daily_bucket(input.bucket_day_epoch_seconds)?;
    let extracted_projections =
        project_m7_owner_billing_daily_projection(input.owner_did.as_str(), owner_points);
    let extracted_report = reconcile_data_layer_m7_owner_billing_daily(
        &extracted_projections,
        ExtractedBillingInput {
            owner_did: input.owner_did.clone(),
            bucket_day_epoch_seconds: input.bucket_day_epoch_seconds,
            messages_stored_total: input.messages_stored_total,
            bytes_stored_total: input.bytes_stored_total,
            queries_executed_total: input.queries_executed_total,
            embeddings_generated_total: input.embeddings_generated_total,
        },
    )
    .map_err(map_m7_billing_error_to_timeseries)?;
    Ok(core_m7_billing_reconciliation_report(extracted_report))
}

pub(crate) fn evaluate_owner_observability(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    query: DataLayerM7BillingQuery,
    profile: ObservabilitySloProfile,
) -> Result<DataLayerM7OwnerObservabilityReport, DataLayerM7TimeseriesError> {
    authorize_owner_scope(
        query.requester_owner_did.as_str(),
        query.owner_did.as_str(),
        DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
    )?;
    let mut monitor = ObservabilityMonitor::new(profile);
    let mut reports = Vec::with_capacity(owner_points.len());
    for point in owner_points {
        let sample = data_layer_m7_project_observability_sample(point);
        let report = monitor
            .evaluate(sample)
            .map_err(map_observability_error_to_timeseries)?;
        reports.push(report);
    }
    Ok(DataLayerM7OwnerObservabilityReport {
        owner_did: query.owner_did,
        reports,
        snapshot: monitor.snapshot(),
    })
}
