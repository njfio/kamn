use kamn_data_layer::{
    project_data_layer_m7_owner_billing_daily, reconcile_data_layer_m7_owner_billing_daily,
    DataLayerM7BillingDailyProjection, DataLayerM7BillingProjectionSampleInput,
    DataLayerM7BillingReconciliationDecision, DataLayerM7BillingReconciliationError,
    DataLayerM7BillingReconciliationInput, DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE,
};

fn billing_sample(
    bucket_day_epoch_seconds: u64,
    message_count: u64,
    bytes_stored: u64,
    query_count: u64,
    embedding_count: u64,
) -> DataLayerM7BillingProjectionSampleInput {
    DataLayerM7BillingProjectionSampleInput {
        bucket_day_epoch_seconds,
        message_count,
        bytes_stored,
        query_count,
        embedding_count,
    }
}

#[test]
fn integration_billing_projection_groups_daily_totals_deterministically() {
    let samples = vec![
        billing_sample(1_708_473_600, 5, 100, 2, 1),
        billing_sample(1_708_473_600, 7, 200, 3, 2),
        billing_sample(1_708_560_000, 11, 400, 4, 3),
    ];

    let projections = project_data_layer_m7_owner_billing_daily("kamn:did:owner:alpha", &samples);
    assert_eq!(
        projections,
        vec![
            DataLayerM7BillingDailyProjection {
                owner_did: "kamn:did:owner:alpha".to_owned(),
                bucket_day_epoch_seconds: 1_708_473_600,
                messages_stored_total: 12,
                bytes_stored_total: 300,
                queries_executed_total: 5,
                embeddings_generated_total: 3,
            },
            DataLayerM7BillingDailyProjection {
                owner_did: "kamn:did:owner:alpha".to_owned(),
                bucket_day_epoch_seconds: 1_708_560_000,
                messages_stored_total: 11,
                bytes_stored_total: 400,
                queries_executed_total: 4,
                embeddings_generated_total: 3,
            },
        ]
    );
}

#[test]
fn integration_reconciliation_covers_match_mismatch_and_missing_projection_zero_totals() {
    let projections = vec![DataLayerM7BillingDailyProjection {
        owner_did: "kamn:did:owner:alpha".to_owned(),
        bucket_day_epoch_seconds: 1_708_473_600,
        messages_stored_total: 12,
        bytes_stored_total: 300,
        queries_executed_total: 5,
        embeddings_generated_total: 3,
    }];

    let matched = reconcile_data_layer_m7_owner_billing_daily(
        &projections,
        DataLayerM7BillingReconciliationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: 1_708_473_600,
            messages_stored_total: 12,
            bytes_stored_total: 300,
            queries_executed_total: 5,
            embeddings_generated_total: 3,
        },
    )
    .expect("matching statement should reconcile");
    assert_eq!(
        matched.decision,
        DataLayerM7BillingReconciliationDecision::Match
    );
    assert_eq!(
        matched.reason_code,
        DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE
    );

    let mismatched = reconcile_data_layer_m7_owner_billing_daily(
        &projections,
        DataLayerM7BillingReconciliationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: 1_708_473_600,
            messages_stored_total: 13,
            bytes_stored_total: 300,
            queries_executed_total: 5,
            embeddings_generated_total: 3,
        },
    )
    .expect("mismatched statement should still reconcile deterministically");
    assert_eq!(
        mismatched.decision,
        DataLayerM7BillingReconciliationDecision::Mismatch
    );
    assert_eq!(
        mismatched.reason_code,
        DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE
    );

    let missing_projection = reconcile_data_layer_m7_owner_billing_daily(
        &projections,
        DataLayerM7BillingReconciliationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: 1_708_560_000,
            messages_stored_total: 0,
            bytes_stored_total: 0,
            queries_executed_total: 0,
            embeddings_generated_total: 0,
        },
    )
    .expect("missing projection day should reconcile against zero totals");
    assert_eq!(
        missing_projection.decision,
        DataLayerM7BillingReconciliationDecision::Match
    );
    assert_eq!(missing_projection.projected_messages_stored_total, 0);
    assert_eq!(missing_projection.projected_bytes_stored_total, 0);
    assert_eq!(missing_projection.projected_queries_executed_total, 0);
    assert_eq!(missing_projection.projected_embeddings_generated_total, 0);
}

#[test]
fn integration_reconciliation_fails_closed_for_invalid_bucket() {
    let error = reconcile_data_layer_m7_owner_billing_daily(
        &[],
        DataLayerM7BillingReconciliationInput {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            bucket_day_epoch_seconds: 1_708_473_601,
            messages_stored_total: 0,
            bytes_stored_total: 0,
            queries_executed_total: 0,
            embeddings_generated_total: 0,
        },
    )
    .expect_err("non-daily-aligned bucket should fail closed");
    assert_eq!(
        error,
        DataLayerM7BillingReconciliationError::InvalidBucketDayEpochSeconds(1_708_473_601)
    );
}
