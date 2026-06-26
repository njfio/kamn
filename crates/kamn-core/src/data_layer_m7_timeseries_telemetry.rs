//! M7 time-series telemetry contracts for ingest, aggregates, and owner billing.
//!
//! This module models PRD M7 behavior as deterministic Rust contracts:
//! owner/agent-scoped telemetry ingest, hourly/daily rollups, network summaries,
//! and owner billing daily usage projection.

mod aggregate_queries;
mod aggregates;
mod billing;
mod errors;
mod models;
mod observability;
mod owner_reports;
mod registry;
mod support;
#[cfg(test)]
mod tests;

pub use aggregates::{
    DataLayerM7AgentDailyAggregate, DataLayerM7AgentHourlyAggregate,
    DataLayerM7BillingReconciliationReport, DataLayerM7NetworkHourlyAggregate,
    DataLayerM7OwnerBillingDailyProjection,
};
pub use errors::DataLayerM7TimeseriesError;
pub use models::{
    DataLayerM7BillingQuery, DataLayerM7BillingReconciliationDecision,
    DataLayerM7BillingReconciliationInput, DataLayerM7OwnerObservabilityReport,
    DataLayerM7TelemetryPointInput, DataLayerM7TelemetryPointRecord,
    DataLayerM7TelemetryScopeQuery, DATA_LAYER_M7_AGGREGATE_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE,
    DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE, DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
    DATA_LAYER_M7_HOURLY_BUCKET_SECONDS, DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE,
    DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
};
pub use observability::data_layer_m7_project_observability_sample;
pub use registry::DataLayerM7TelemetryRegistry;

pub(crate) use aggregate_queries::{
    aggregate_agent_daily, aggregate_agent_hourly, aggregate_network_hourly,
};
pub(crate) use billing::{
    core_m7_billing_reconciliation_report, map_m7_billing_error_to_timeseries,
    project_m7_owner_billing_daily_projection, project_m7_owner_billing_daily_rows,
};
pub(crate) use observability::{
    map_observability_error_to_timeseries, project_m7_observability_projection,
};
pub(crate) use owner_reports::{
    evaluate_owner_observability, project_owner_billing_daily, reconcile_owner_billing_daily,
};
pub(crate) use support::{
    authorize_owner_scope, daily_bucket, hourly_bucket, validate_daily_bucket, validate_kamn_did,
};
