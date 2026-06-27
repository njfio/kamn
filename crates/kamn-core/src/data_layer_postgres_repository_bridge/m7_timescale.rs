use crate::{
    DataLayerM7TelemetryPointRecord, DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
    DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
};

use super::{
    build_requester_session, validate_owner_did, validate_timescale_config,
    DataLayerPgM7TimescaleConfig, DataLayerPgM7TimescaleOwnerRollupRequest,
    DataLayerPgOperationKind, DataLayerPgRepositoryBridgeError, DataLayerPgSqlOperation,
    DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT,
    DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
};

/// Runs the data layer pg project m7 timescale ingest operation contract helper.
pub fn data_layer_pg_project_m7_timescale_ingest_operation(
    record: &DataLayerM7TelemetryPointRecord,
    requester_did: &str,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_timescale_record(record)?;
    validate_bucket_alignment(record)?;
    build_timescale_ingest_operation(requester_did, config.hypertable_name)
}

/// Runs the data layer pg project m7 timescale owner rollup query operation contract helper.
pub fn data_layer_pg_project_m7_timescale_owner_rollup_query_operation(
    request: DataLayerPgM7TimescaleOwnerRollupRequest,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_rollup_request(&request)?;
    let interval_marker = resolve_interval_marker(request.bucket_window_seconds)?;
    let limit = resolve_timescale_limit(request.limit)?;
    build_owner_rollup_operation(
        request.requester_did.as_str(),
        config.hypertable_name,
        interval_marker,
        limit,
    )
}

fn validate_timescale_record(
    record: &DataLayerM7TelemetryPointRecord,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_owner_did(record.owner_did.as_str())?;
    if record.agent_did.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("agent_did"));
    }
    if record.timestamp_epoch_seconds == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "timestamp_epoch_seconds",
        ));
    }
    Ok(())
}

fn validate_bucket_alignment(
    record: &DataLayerM7TelemetryPointRecord,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_bucket(
        record.bucket_hour_epoch_seconds,
        aligned_bucket(
            record.timestamp_epoch_seconds,
            DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
        ),
        DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
    )?;
    validate_bucket(
        record.bucket_day_epoch_seconds,
        aligned_bucket(
            record.timestamp_epoch_seconds,
            DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
        ),
        DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
    )
}

fn aligned_bucket(timestamp_epoch_seconds: u64, bucket_seconds: u64) -> u64 {
    timestamp_epoch_seconds - (timestamp_epoch_seconds % bucket_seconds)
}

fn validate_bucket(
    actual: u64,
    expected: u64,
    bucket_window_seconds: u64,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    if actual != expected {
        return Err(invalid_bucket_window(bucket_window_seconds));
    }
    Ok(())
}

fn build_timescale_ingest_operation(
    requester_did: &str,
    hypertable_name: String,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertTelemetryPoint,
        sql: timescale_ingest_sql(&hypertable_name),
        bind_markers: timescale_ingest_bind_markers(),
        session: build_requester_session(requester_did)?,
    })
}

fn timescale_ingest_sql(hypertable_name: &str) -> String {
    format!(
        "INSERT INTO {hypertable_name} (owner_did, agent_did, observed_at, bucket_hour_epoch_seconds, bucket_day_epoch_seconds, message_count, bytes_stored, query_count, embedding_count, embedding_anomaly_count, ingress_latency_ms_p95, egress_latency_ms_p95, active_sessions, sequence) VALUES ($1, $2, to_timestamp($3), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14);"
    )
}

fn timescale_ingest_bind_markers() -> Vec<&'static str> {
    vec![
        "owner_did",
        "agent_did",
        "timestamp_epoch_seconds",
        "bucket_hour_epoch_seconds",
        "bucket_day_epoch_seconds",
        "message_count",
        "bytes_stored",
        "query_count",
        "embedding_count",
        "embedding_anomaly_count",
        "ingress_latency_ms_p95",
        "egress_latency_ms_p95",
        "active_sessions",
        "sequence",
    ]
}

fn validate_rollup_request(
    request: &DataLayerPgM7TimescaleOwnerRollupRequest,
) -> Result<(), DataLayerPgRepositoryBridgeError> {
    validate_owner_did(request.query.owner_did.as_str())?;
    validate_owner_did(request.query.requester_owner_did.as_str())
}

fn resolve_interval_marker(
    bucket_window_seconds: u64,
) -> Result<&'static str, DataLayerPgRepositoryBridgeError> {
    match bucket_window_seconds {
        DATA_LAYER_M7_HOURLY_BUCKET_SECONDS => Ok("1 hour"),
        DATA_LAYER_M7_DAILY_BUCKET_SECONDS => Ok("1 day"),
        other => Err(invalid_bucket_window(other)),
    }
}

fn resolve_timescale_limit(
    limit: Option<usize>,
) -> Result<usize, DataLayerPgRepositoryBridgeError> {
    let limit = limit.unwrap_or(DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT as u32,
        });
    }
    Ok(limit)
}

fn build_owner_rollup_operation(
    requester_did: &str,
    hypertable_name: String,
    interval_marker: &'static str,
    _limit: usize,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::QueryTelemetryOwnerRollup,
        sql: owner_rollup_sql(interval_marker, &hypertable_name),
        bind_markers: vec!["owner_did", "limit"],
        session: build_requester_session(requester_did)?,
    })
}

fn owner_rollup_sql(interval_marker: &str, hypertable_name: &str) -> String {
    format!(
        "SELECT time_bucket(INTERVAL '{interval_marker}', observed_at) AS bucket_start, SUM(message_count) AS message_count_total, SUM(bytes_stored) AS bytes_stored_total, SUM(query_count) AS query_count_total, SUM(embedding_count) AS embedding_count_total FROM {hypertable_name} WHERE owner_did = $1 GROUP BY bucket_start ORDER BY bucket_start DESC LIMIT $2;"
    )
}

fn invalid_bucket_window(bucket_window_seconds: u64) -> DataLayerPgRepositoryBridgeError {
    DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
        reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
        bucket_window_seconds,
    }
}
