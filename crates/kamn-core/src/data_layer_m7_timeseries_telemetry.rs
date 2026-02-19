//! M7 time-series telemetry contracts for ingest, aggregates, and owner billing.
//!
//! This module models PRD M7 behavior as deterministic Rust contracts:
//! owner/agent-scoped telemetry ingest, hourly/daily rollups, network summaries,
//! and owner billing daily usage projection.

use crate::{
    ObservabilityError, ObservabilityMonitor, ObservabilityReport, ObservabilitySample,
    ObservabilitySloProfile, ObservabilitySnapshot,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

type DataLayerM7NetworkHourlyAccumulator = (u64, u64, u64, u64, u64, BTreeSet<String>);

/// Hourly bucket size in seconds.
pub const DATA_LAYER_M7_HOURLY_BUCKET_SECONDS: u64 = 3_600;
/// Daily bucket size in seconds.
pub const DATA_LAYER_M7_DAILY_BUCKET_SECONDS: u64 = 86_400;
/// Stable reason marker for successful aggregate results.
pub const DATA_LAYER_M7_AGGREGATE_REASON_CODE: &str = "m7_timeseries_aggregate_computed";
/// Stable reason marker for owner-scope authorization failures.
pub const DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE: &str = "m7_timeseries_owner_scope_denied";
/// Stable reason marker for successful billing reconciliation match.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_match";
/// Stable reason marker for billing reconciliation mismatch.
pub const DATA_LAYER_M7_BILLING_RECONCILIATION_MISMATCH_REASON_CODE: &str =
    "m7_billing_reconciliation_mismatch";
/// Stable reason marker for invalid projected observability samples.
pub const DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE: &str =
    "m7_observability_sample_invalid";

/// Input payload for one telemetry point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryPointInput {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID scope.
    pub agent_did: String,
    /// Observation timestamp (epoch seconds).
    pub timestamp_epoch_seconds: u64,
    /// Message count for this sample.
    pub message_count: u64,
    /// Bytes stored for this sample.
    pub bytes_stored: u64,
    /// Query count for this sample.
    pub query_count: u64,
    /// Embedding generation count for this sample.
    pub embedding_count: u64,
    /// Embedding anomaly count for this sample.
    pub embedding_anomaly_count: u64,
    /// P95 ingress latency in ms.
    pub ingress_latency_ms_p95: u32,
    /// P95 egress latency in ms.
    pub egress_latency_ms_p95: u32,
    /// Active session count at sample time.
    pub active_sessions: u32,
}

/// Stored telemetry record with derived bucket markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryPointRecord {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID scope.
    pub agent_did: String,
    /// Observation timestamp (epoch seconds).
    pub timestamp_epoch_seconds: u64,
    /// Hourly bucket start (epoch seconds).
    pub bucket_hour_epoch_seconds: u64,
    /// Daily bucket start (epoch seconds).
    pub bucket_day_epoch_seconds: u64,
    /// Message count for this sample.
    pub message_count: u64,
    /// Bytes stored for this sample.
    pub bytes_stored: u64,
    /// Query count for this sample.
    pub query_count: u64,
    /// Embedding generation count for this sample.
    pub embedding_count: u64,
    /// Embedding anomaly count for this sample.
    pub embedding_anomaly_count: u64,
    /// P95 ingress latency in ms.
    pub ingress_latency_ms_p95: u32,
    /// P95 egress latency in ms.
    pub egress_latency_ms_p95: u32,
    /// Active session count at sample time.
    pub active_sessions: u32,
    /// Append-order sequence.
    pub sequence: u64,
}

/// Scope query for owner+agent aggregate views.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7TelemetryScopeQuery {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Agent DID.
    pub agent_did: String,
}

/// Owner billing projection query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingQuery {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
}

/// Owner daily billing statement input used for reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationInput {
    /// Requester owner DID.
    pub requester_owner_did: String,
    /// Target owner DID.
    pub owner_did: String,
    /// Billing day bucket start (epoch seconds, aligned to day boundary).
    pub bucket_day_epoch_seconds: u64,
    /// Statement metric: messages stored.
    pub messages_stored_total: u64,
    /// Statement metric: bytes stored.
    pub bytes_stored_total: u64,
    /// Statement metric: queries executed.
    pub queries_executed_total: u64,
    /// Statement metric: embeddings generated.
    pub embeddings_generated_total: u64,
}

/// Billing reconciliation decision output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM7BillingReconciliationDecision {
    /// Statement equals projected owner daily totals.
    Match,
    /// Statement differs from projected owner daily totals.
    Mismatch,
}

/// Deterministic billing reconciliation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7BillingReconciliationReport {
    /// Owner DID scope.
    pub owner_did: String,
    /// Billing day bucket start.
    pub bucket_day_epoch_seconds: u64,
    /// Reconciliation decision.
    pub decision: DataLayerM7BillingReconciliationDecision,
    /// Stable decision reason marker.
    pub reason_code: &'static str,
    /// Projected metric: messages stored.
    pub projected_messages_stored_total: u64,
    /// Projected metric: bytes stored.
    pub projected_bytes_stored_total: u64,
    /// Projected metric: queries executed.
    pub projected_queries_executed_total: u64,
    /// Projected metric: embeddings generated.
    pub projected_embeddings_generated_total: u64,
    /// Statement metric: messages stored.
    pub statement_messages_stored_total: u64,
    /// Statement metric: bytes stored.
    pub statement_bytes_stored_total: u64,
    /// Statement metric: queries executed.
    pub statement_queries_executed_total: u64,
    /// Statement metric: embeddings generated.
    pub statement_embeddings_generated_total: u64,
}

/// Owner-scoped observability evaluation output derived from M7 telemetry points.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM7OwnerObservabilityReport {
    /// Owner DID scope.
    pub owner_did: String,
    /// Ordered observability reports per owner telemetry point.
    pub reports: Vec<ObservabilityReport>,
    /// Rolling snapshot derived from the report history.
    pub snapshot: ObservabilitySnapshot,
}

/// Hourly aggregate row for one owner+agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7AgentHourlyAggregate {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID scope.
    pub agent_did: String,
    /// Hourly bucket start (epoch seconds).
    pub bucket_hour_epoch_seconds: u64,
    /// Total messages in bucket.
    pub message_count_total: u64,
    /// Total bytes in bucket.
    pub bytes_stored_total: u64,
    /// Total queries in bucket.
    pub query_count_total: u64,
    /// Total embeddings in bucket.
    pub embedding_count_total: u64,
    /// Total embedding anomalies in bucket.
    pub embedding_anomaly_count_total: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Daily aggregate row for one owner+agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7AgentDailyAggregate {
    /// Owner DID scope.
    pub owner_did: String,
    /// Agent DID scope.
    pub agent_did: String,
    /// Daily bucket start (epoch seconds).
    pub bucket_day_epoch_seconds: u64,
    /// Total messages in bucket.
    pub message_count_total: u64,
    /// Total bytes in bucket.
    pub bytes_stored_total: u64,
    /// Total queries in bucket.
    pub query_count_total: u64,
    /// Total embeddings in bucket.
    pub embedding_count_total: u64,
    /// Total embedding anomalies in bucket.
    pub embedding_anomaly_count_total: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Hourly network summary aggregate row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7NetworkHourlyAggregate {
    /// Hourly bucket start (epoch seconds).
    pub bucket_hour_epoch_seconds: u64,
    /// Total messages in bucket.
    pub message_count_total: u64,
    /// Total bytes in bucket.
    pub bytes_stored_total: u64,
    /// Total queries in bucket.
    pub query_count_total: u64,
    /// Total embeddings in bucket.
    pub embedding_count_total: u64,
    /// Total embedding anomalies in bucket.
    pub embedding_anomaly_count_total: u64,
    /// Unique active agent count in bucket.
    pub active_agent_count: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// Owner billing daily projection row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM7OwnerBillingDailyProjection {
    /// Owner DID scope.
    pub owner_did: String,
    /// Daily bucket start (epoch seconds).
    pub bucket_day_epoch_seconds: u64,
    /// Billing metric: messages stored.
    pub messages_stored_total: u64,
    /// Billing metric: bytes stored.
    pub bytes_stored_total: u64,
    /// Billing metric: queries executed.
    pub queries_executed_total: u64,
    /// Billing metric: embeddings generated.
    pub embeddings_generated_total: u64,
    /// Stable reason marker.
    pub reason_code: &'static str,
}

/// M7 telemetry registry and aggregate query service.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM7TelemetryRegistry {
    points_by_owner: BTreeMap<String, Vec<DataLayerM7TelemetryPointRecord>>,
}

impl DataLayerM7TelemetryRegistry {
    /// Creates an empty telemetry registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingests one telemetry sample point.
    pub fn ingest_point(
        &mut self,
        input: DataLayerM7TelemetryPointInput,
    ) -> Result<DataLayerM7TelemetryPointRecord, DataLayerM7TimeseriesError> {
        validate_kamn_did(input.owner_did.as_str())?;
        validate_kamn_did(input.agent_did.as_str())?;
        if input.timestamp_epoch_seconds == 0 {
            return Err(DataLayerM7TimeseriesError::EmptyField(
                "timestamp_epoch_seconds",
            ));
        }

        let owner_points = self
            .points_by_owner
            .entry(input.owner_did.clone())
            .or_default();
        let record = DataLayerM7TelemetryPointRecord {
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
            sequence: owner_points.len() as u64 + 1,
        };
        owner_points.push(record.clone());
        Ok(record)
    }

    /// Returns all telemetry points for one owner in append order.
    pub fn points_for_owner(&self, owner_did: &str) -> Option<&[DataLayerM7TelemetryPointRecord]> {
        self.points_by_owner.get(owner_did).map(Vec::as_slice)
    }

    /// Computes owner+agent hourly rollups.
    pub fn aggregate_agent_hourly(
        &self,
        query: DataLayerM7TelemetryScopeQuery,
    ) -> Result<Vec<DataLayerM7AgentHourlyAggregate>, DataLayerM7TimeseriesError> {
        authorize_owner_scope(
            query.requester_owner_did.as_str(),
            query.owner_did.as_str(),
            DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        )?;
        validate_kamn_did(query.agent_did.as_str())?;
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;

        let mut grouped: BTreeMap<u64, (u64, u64, u64, u64, u64)> = BTreeMap::new();
        for point in owner_points
            .iter()
            .filter(|point| point.agent_did == query.agent_did)
        {
            let entry = grouped
                .entry(point.bucket_hour_epoch_seconds)
                .or_insert((0, 0, 0, 0, 0));
            entry.0 += point.message_count;
            entry.1 += point.bytes_stored;
            entry.2 += point.query_count;
            entry.3 += point.embedding_count;
            entry.4 += point.embedding_anomaly_count;
        }

        Ok(grouped
            .into_iter()
            .map(
                |(
                    bucket_hour_epoch_seconds,
                    (
                        message_count_total,
                        bytes_stored_total,
                        query_count_total,
                        embedding_count_total,
                        embedding_anomaly_count_total,
                    ),
                )| DataLayerM7AgentHourlyAggregate {
                    owner_did: query.owner_did.clone(),
                    agent_did: query.agent_did.clone(),
                    bucket_hour_epoch_seconds,
                    message_count_total,
                    bytes_stored_total,
                    query_count_total,
                    embedding_count_total,
                    embedding_anomaly_count_total,
                    reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
                },
            )
            .collect())
    }

    /// Computes owner+agent daily rollups.
    pub fn aggregate_agent_daily(
        &self,
        query: DataLayerM7TelemetryScopeQuery,
    ) -> Result<Vec<DataLayerM7AgentDailyAggregate>, DataLayerM7TimeseriesError> {
        authorize_owner_scope(
            query.requester_owner_did.as_str(),
            query.owner_did.as_str(),
            DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        )?;
        validate_kamn_did(query.agent_did.as_str())?;
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;

        let mut grouped: BTreeMap<u64, (u64, u64, u64, u64, u64)> = BTreeMap::new();
        for point in owner_points
            .iter()
            .filter(|point| point.agent_did == query.agent_did)
        {
            let entry = grouped
                .entry(point.bucket_day_epoch_seconds)
                .or_insert((0, 0, 0, 0, 0));
            entry.0 += point.message_count;
            entry.1 += point.bytes_stored;
            entry.2 += point.query_count;
            entry.3 += point.embedding_count;
            entry.4 += point.embedding_anomaly_count;
        }

        Ok(grouped
            .into_iter()
            .map(
                |(
                    bucket_day_epoch_seconds,
                    (
                        message_count_total,
                        bytes_stored_total,
                        query_count_total,
                        embedding_count_total,
                        embedding_anomaly_count_total,
                    ),
                )| DataLayerM7AgentDailyAggregate {
                    owner_did: query.owner_did.clone(),
                    agent_did: query.agent_did.clone(),
                    bucket_day_epoch_seconds,
                    message_count_total,
                    bytes_stored_total,
                    query_count_total,
                    embedding_count_total,
                    embedding_anomaly_count_total,
                    reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
                },
            )
            .collect())
    }

    /// Computes network-wide hourly summary across all owners.
    pub fn aggregate_network_hourly(
        &self,
    ) -> Result<Vec<DataLayerM7NetworkHourlyAggregate>, DataLayerM7TimeseriesError> {
        let mut grouped: BTreeMap<u64, DataLayerM7NetworkHourlyAccumulator> = BTreeMap::new();
        for owner_points in self.points_by_owner.values() {
            for point in owner_points {
                let entry = grouped.entry(point.bucket_hour_epoch_seconds).or_insert((
                    0,
                    0,
                    0,
                    0,
                    0,
                    BTreeSet::new(),
                ));
                entry.0 += point.message_count;
                entry.1 += point.bytes_stored;
                entry.2 += point.query_count;
                entry.3 += point.embedding_count;
                entry.4 += point.embedding_anomaly_count;
                entry.5.insert(point.agent_did.clone());
            }
        }

        Ok(grouped
            .into_iter()
            .map(
                |(
                    bucket_hour_epoch_seconds,
                    (
                        message_count_total,
                        bytes_stored_total,
                        query_count_total,
                        embedding_count_total,
                        embedding_anomaly_count_total,
                        active_agents,
                    ),
                )| DataLayerM7NetworkHourlyAggregate {
                    bucket_hour_epoch_seconds,
                    message_count_total,
                    bytes_stored_total,
                    query_count_total,
                    embedding_count_total,
                    embedding_anomaly_count_total,
                    active_agent_count: active_agents.len() as u64,
                    reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
                },
            )
            .collect())
    }

    /// Projects owner billing daily usage rows.
    pub fn project_owner_billing_daily(
        &self,
        query: DataLayerM7BillingQuery,
    ) -> Result<Vec<DataLayerM7OwnerBillingDailyProjection>, DataLayerM7TimeseriesError> {
        authorize_owner_scope(
            query.requester_owner_did.as_str(),
            query.owner_did.as_str(),
            DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        )?;
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;

        let mut grouped: BTreeMap<u64, (u64, u64, u64, u64)> = BTreeMap::new();
        for point in owner_points {
            let entry = grouped
                .entry(point.bucket_day_epoch_seconds)
                .or_insert((0, 0, 0, 0));
            entry.0 += point.message_count;
            entry.1 += point.bytes_stored;
            entry.2 += point.query_count;
            entry.3 += point.embedding_count;
        }

        Ok(grouped
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
                )| DataLayerM7OwnerBillingDailyProjection {
                    owner_did: query.owner_did.clone(),
                    bucket_day_epoch_seconds,
                    messages_stored_total,
                    bytes_stored_total,
                    queries_executed_total,
                    embeddings_generated_total,
                    reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
                },
            )
            .collect())
    }

    /// Reconciles one owner daily billing statement against projected totals.
    pub fn reconcile_owner_billing_daily(
        &self,
        input: DataLayerM7BillingReconciliationInput,
    ) -> Result<DataLayerM7BillingReconciliationReport, DataLayerM7TimeseriesError> {
        authorize_owner_scope(
            input.requester_owner_did.as_str(),
            input.owner_did.as_str(),
            DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        )?;
        if input.bucket_day_epoch_seconds == 0
            || !input
                .bucket_day_epoch_seconds
                .is_multiple_of(DATA_LAYER_M7_DAILY_BUCKET_SECONDS)
        {
            return Err(DataLayerM7TimeseriesError::InvalidBucketDayEpochSeconds(
                input.bucket_day_epoch_seconds,
            ));
        }

        let projections = self.project_owner_billing_daily(DataLayerM7BillingQuery {
            requester_owner_did: input.requester_owner_did.clone(),
            owner_did: input.owner_did.clone(),
        })?;
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

    /// Evaluates owner telemetry points through canonical observability contracts.
    pub fn evaluate_owner_observability(
        &self,
        query: DataLayerM7BillingQuery,
        profile: ObservabilitySloProfile,
    ) -> Result<DataLayerM7OwnerObservabilityReport, DataLayerM7TimeseriesError> {
        authorize_owner_scope(
            query.requester_owner_did.as_str(),
            query.owner_did.as_str(),
            DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
        )?;
        let owner_points = self.owner_points_or_error(query.owner_did.as_str())?;
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

    fn owner_points_or_error(
        &self,
        owner_did: &str,
    ) -> Result<&[DataLayerM7TelemetryPointRecord], DataLayerM7TimeseriesError> {
        validate_kamn_did(owner_did)?;
        self.points_by_owner
            .get(owner_did)
            .map(Vec::as_slice)
            .ok_or_else(|| DataLayerM7TimeseriesError::OwnerNotFound {
                owner_did: owner_did.to_owned(),
            })
    }
}

/// Error taxonomy for M7 time-series contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM7TimeseriesError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID failed validation.
    InvalidDid(String),
    /// Owner scope was not found.
    OwnerNotFound {
        /// Missing owner DID.
        owner_did: String,
    },
    /// Owner-scope authorization failed.
    OwnerScopeViolation {
        /// Stable reason marker.
        reason_code: &'static str,
    },
    /// Billing day bucket epoch is zero or not daily-aligned.
    InvalidBucketDayEpochSeconds(u64),
    /// Observability projection produced an invalid sample.
    ObservabilitySampleInvalid {
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerM7TimeseriesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::OwnerNotFound { owner_did } => write!(f, "owner not found: {owner_did}"),
            Self::OwnerScopeViolation { reason_code } => {
                write!(f, "owner scope violation: {reason_code}")
            }
            Self::InvalidBucketDayEpochSeconds(value) => {
                write!(f, "invalid billing day bucket epoch: {value}")
            }
            Self::ObservabilitySampleInvalid { reason_code } => {
                write!(f, "invalid observability sample projection: {reason_code}")
            }
        }
    }
}

impl std::error::Error for DataLayerM7TimeseriesError {}

/// Projects one M7 telemetry point into a canonical observability sample.
pub fn data_layer_m7_project_observability_sample(
    point: &DataLayerM7TelemetryPointRecord,
) -> ObservabilitySample {
    let latency_p50_ms = u64::from(point.ingress_latency_ms_p95);
    let latency_p99_ms = u64::from(
        point
            .ingress_latency_ms_p95
            .max(point.egress_latency_ms_p95),
    );
    let throughput_tps = derive_observability_throughput_tps(point);
    let error_rate_pct = derive_observability_error_rate_pct(point);
    let availability_pct = derive_observability_availability_pct(point.active_sessions);

    ObservabilitySample {
        latency_p50_ms,
        latency_p99_ms,
        throughput_tps,
        error_rate_pct,
        availability_pct,
        timestamp_epoch_s: point.timestamp_epoch_seconds,
    }
}

fn map_observability_error_to_timeseries(_error: ObservabilityError) -> DataLayerM7TimeseriesError {
    DataLayerM7TimeseriesError::ObservabilitySampleInvalid {
        reason_code: DATA_LAYER_M7_OBSERVABILITY_SAMPLE_INVALID_REASON_CODE,
    }
}

fn derive_observability_throughput_tps(point: &DataLayerM7TelemetryPointRecord) -> u64 {
    let activity = point
        .message_count
        .saturating_add(point.query_count)
        .saturating_add(point.embedding_count);
    let session_boost = u64::from(point.active_sessions).saturating_mul(1_000);
    activity.saturating_add(session_boost).max(1)
}

fn derive_observability_error_rate_pct(point: &DataLayerM7TelemetryPointRecord) -> f64 {
    if point.embedding_count == 0 {
        return 0.0;
    }
    let ratio = (point.embedding_anomaly_count as f64 / point.embedding_count as f64) * 100.0;
    ratio.clamp(0.0, 100.0)
}

fn derive_observability_availability_pct(active_sessions: u32) -> f64 {
    if active_sessions == 0 {
        0.0
    } else {
        100.0
    }
}

fn validate_kamn_did(value: &str) -> Result<(), DataLayerM7TimeseriesError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.starts_with("kamn:did:") {
        return Err(DataLayerM7TimeseriesError::InvalidDid(value.to_owned()));
    }
    let segments = trimmed.split(':').collect::<Vec<_>>();
    if segments.len() < 4 || segments.iter().any(|segment| segment.is_empty()) {
        return Err(DataLayerM7TimeseriesError::InvalidDid(value.to_owned()));
    }
    Ok(())
}

fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
    reason_code: &'static str,
) -> Result<(), DataLayerM7TimeseriesError> {
    validate_kamn_did(requester_owner_did)?;
    validate_kamn_did(owner_did)?;
    if requester_owner_did != owner_did {
        return Err(DataLayerM7TimeseriesError::OwnerScopeViolation { reason_code });
    }
    Ok(())
}

fn hourly_bucket(timestamp_epoch_seconds: u64) -> u64 {
    timestamp_epoch_seconds - (timestamp_epoch_seconds % DATA_LAYER_M7_HOURLY_BUCKET_SECONDS)
}

fn daily_bucket(timestamp_epoch_seconds: u64) -> u64 {
    timestamp_epoch_seconds - (timestamp_epoch_seconds % DATA_LAYER_M7_DAILY_BUCKET_SECONDS)
}
