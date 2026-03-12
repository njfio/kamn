use super::{
    aggregate_agent_daily, aggregate_agent_hourly, aggregate_network_hourly,
    daily_bucket, evaluate_owner_observability, hourly_bucket, project_owner_billing_daily,
    reconcile_owner_billing_daily, validate_daily_bucket, validate_kamn_did,
    DataLayerM7AgentDailyAggregate,
    DataLayerM7AgentHourlyAggregate, DataLayerM7BillingQuery,
    DataLayerM7BillingReconciliationInput, DataLayerM7BillingReconciliationReport,
    DataLayerM7NetworkHourlyAggregate, DataLayerM7OwnerBillingDailyProjection,
    DataLayerM7OwnerObservabilityReport, DataLayerM7TelemetryPointInput,
    DataLayerM7TelemetryPointRecord, DataLayerM7TelemetryScopeQuery, DataLayerM7TimeseriesError,
};
use crate::ObservabilitySloProfile;
use std::collections::BTreeMap;

/// M7 telemetry registry and aggregate query service.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM7TelemetryRegistry {
    points_by_owner: BTreeMap<String, Vec<DataLayerM7TelemetryPointRecord>>,
}

impl DataLayerM7TelemetryRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest_point(
        &mut self,
        input: DataLayerM7TelemetryPointInput,
    ) -> Result<DataLayerM7TelemetryPointRecord, DataLayerM7TimeseriesError> {
        validate_ingest_input(&input)?;
        let owner_points = self.points_by_owner.entry(input.owner_did.clone()).or_default();
        let record = build_record(input, owner_points.len() as u64 + 1);
        owner_points.push(record.clone());
        Ok(record)
    }

    pub fn points_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM7TelemetryPointRecord]> {
        self.points_by_owner.get(owner_did).map(Vec::as_slice)
    }

    pub fn aggregate_agent_hourly(
        &self,
        query: DataLayerM7TelemetryScopeQuery,
    ) -> Result<Vec<DataLayerM7AgentHourlyAggregate>, DataLayerM7TimeseriesError> {
        aggregate_agent_hourly(self.owner_points_or_error(query.owner_did.as_str())?, &query)
    }

    pub fn aggregate_agent_daily(
        &self,
        query: DataLayerM7TelemetryScopeQuery,
    ) -> Result<Vec<DataLayerM7AgentDailyAggregate>, DataLayerM7TimeseriesError> {
        aggregate_agent_daily(self.owner_points_or_error(query.owner_did.as_str())?, &query)
    }

    pub fn aggregate_network_hourly(
        &self,
    ) -> Result<Vec<DataLayerM7NetworkHourlyAggregate>, DataLayerM7TimeseriesError> {
        Ok(aggregate_network_hourly(&self.points_by_owner))
    }

    pub fn project_owner_billing_daily(
        &self,
        query: DataLayerM7BillingQuery,
    ) -> Result<Vec<DataLayerM7OwnerBillingDailyProjection>, DataLayerM7TimeseriesError> {
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;
        project_owner_billing_daily(owner_points, query)
    }

    pub fn reconcile_owner_billing_daily(
        &self,
        input: DataLayerM7BillingReconciliationInput,
    ) -> Result<DataLayerM7BillingReconciliationReport, DataLayerM7TimeseriesError> {
        validate_daily_bucket(input.bucket_day_epoch_seconds)?;
        let owner_points = self.owner_points_or_error(input.owner_did.as_str())?;
        reconcile_owner_billing_daily(owner_points, input)
    }

    pub fn evaluate_owner_observability(
        &self,
        query: DataLayerM7BillingQuery,
        profile: ObservabilitySloProfile,
    ) -> Result<DataLayerM7OwnerObservabilityReport, DataLayerM7TimeseriesError> {
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;
        evaluate_owner_observability(owner_points, query, profile)
    }

    fn owner_points_or_error(
        &self,
        owner_did: &str,
    ) -> Result<&[DataLayerM7TelemetryPointRecord], DataLayerM7TimeseriesError> {
        validate_kamn_did(owner_did)?;
        self.points_by_owner.get(owner_did).map(Vec::as_slice).ok_or_else(|| {
            DataLayerM7TimeseriesError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            }
        })
    }
}

fn validate_ingest_input(
    input: &DataLayerM7TelemetryPointInput,
) -> Result<(), DataLayerM7TimeseriesError> {
    validate_kamn_did(input.owner_did.as_str())?;
    validate_kamn_did(input.agent_did.as_str())?;
    if input.timestamp_epoch_seconds == 0 {
        return Err(DataLayerM7TimeseriesError::EmptyField("timestamp_epoch_seconds"));
    }
    Ok(())
}

fn build_record(input: DataLayerM7TelemetryPointInput, sequence: u64) -> DataLayerM7TelemetryPointRecord {
    DataLayerM7TelemetryPointRecord {
        owner_did: input.owner_did,
        agent_did: input.agent_did,
        timestamp_epoch_seconds: input.timestamp_epoch_seconds,
        bucket_hour_epoch_seconds: hourly_bucket(input.timestamp_epoch_seconds),
        bucket_day_epoch_seconds: daily_bucket(input.timestamp_epoch_seconds),
        message_count: input.message_count,
        bytes_stored: input.bytes_stored,
        query_count: input.query_count,
        embedding_count: input.embedding_count,
        embedding_anomaly_count: input.embedding_anomaly_count,
        ingress_latency_ms_p95: input.ingress_latency_ms_p95,
        egress_latency_ms_p95: input.egress_latency_ms_p95,
        active_sessions: input.active_sessions,
        sequence,
    }
}
