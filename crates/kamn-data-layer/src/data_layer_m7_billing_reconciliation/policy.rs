use std::collections::BTreeMap;

use super::{
    DataLayerM7BillingDailyProjection, DataLayerM7BillingProjectionSampleInput,
    DataLayerM7BillingReconciliationDecision, DataLayerM7BillingReconciliationError,
    DataLayerM7BillingReconciliationInput, DataLayerM7BillingReconciliationReport,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE,
};

const DATA_LAYER_M7_DAILY_BUCKET_SECONDS: u64 = 86_400;

/// Projects owner daily billing totals from deterministic M7 billing samples.
pub fn project_data_layer_m7_owner_billing_daily(
    owner_did: &str,
    samples: &[DataLayerM7BillingProjectionSampleInput],
) -> Vec<DataLayerM7BillingDailyProjection> {
    let mut grouped: BTreeMap<u64, (u64, u64, u64, u64)> = BTreeMap::new();
    for sample in samples {
        let entry = grouped
            .entry(sample.bucket_day_epoch_seconds)
            .or_insert((0, 0, 0, 0));
        entry.0 += sample.message_count;
        entry.1 += sample.bytes_stored;
        entry.2 += sample.query_count;
        entry.3 += sample.embedding_count;
    }

    grouped
        .into_iter()
        .map(
            |(
                bucket_day_epoch_seconds,
                (
                    messages_stored_total,
                    bytes_stored_total,
                    queries_executed_total,
                    embeddings_generated_total,
                ),
            )| DataLayerM7BillingDailyProjection {
                owner_did: owner_did.to_owned(),
                bucket_day_epoch_seconds,
                messages_stored_total,
                bytes_stored_total,
                queries_executed_total,
                embeddings_generated_total,
            },
        )
        .collect()
}

/// Reconciles one owner daily billing statement against projected totals.
pub fn reconcile_data_layer_m7_owner_billing_daily(
    projections: &[DataLayerM7BillingDailyProjection],
    input: DataLayerM7BillingReconciliationInput,
) -> Result<DataLayerM7BillingReconciliationReport, DataLayerM7BillingReconciliationError> {
    validate_bucket_day_epoch_seconds(input.bucket_day_epoch_seconds)?;

    let projection = projections
        .iter()
        .find(|entry| entry.bucket_day_epoch_seconds == input.bucket_day_epoch_seconds);
    let projected_messages_stored_total =
        projection.map_or(0, |entry| entry.messages_stored_total);
    let projected_bytes_stored_total = projection.map_or(0, |entry| entry.bytes_stored_total);
    let projected_queries_executed_total =
        projection.map_or(0, |entry| entry.queries_executed_total);
    let projected_embeddings_generated_total =
        projection.map_or(0, |entry| entry.embeddings_generated_total);

    let mismatch = projected_messages_stored_total != input.messages_stored_total
        || projected_bytes_stored_total != input.bytes_stored_total
        || projected_queries_executed_total != input.queries_executed_total
        || projected_embeddings_generated_total != input.embeddings_generated_total;
    let (decision, reason_code) = if mismatch {
        (
            DataLayerM7BillingReconciliationDecision::Mismatch,
            DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE,
        )
    } else {
        (
            DataLayerM7BillingReconciliationDecision::Match,
            DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
        )
    };

    Ok(DataLayerM7BillingReconciliationReport {
        owner_did: input.owner_did,
        bucket_day_epoch_seconds: input.bucket_day_epoch_seconds,
        decision,
        reason_code,
        projected_messages_stored_total,
        projected_bytes_stored_total,
        projected_queries_executed_total,
        projected_embeddings_generated_total,
        statement_messages_stored_total: input.messages_stored_total,
        statement_bytes_stored_total: input.bytes_stored_total,
        statement_queries_executed_total: input.queries_executed_total,
        statement_embeddings_generated_total: input.embeddings_generated_total,
    })
}

fn validate_bucket_day_epoch_seconds(
    bucket_day_epoch_seconds: u64,
) -> Result<(), DataLayerM7BillingReconciliationError> {
    if bucket_day_epoch_seconds == 0
        || !bucket_day_epoch_seconds.is_multiple_of(DATA_LAYER_M7_DAILY_BUCKET_SECONDS)
    {
        return Err(DataLayerM7BillingReconciliationError::InvalidBucketDayEpochSeconds(
            bucket_day_epoch_seconds,
        ));
    }
    Ok(())
}
