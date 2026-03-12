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

pub fn data_layer_pg_project_m7_timescale_ingest_operation(
    record: &DataLayerM7TelemetryPointRecord,
    requester_did: &str,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_owner_did(record.owner_did.as_str())?;
    if record.agent_did.trim().is_empty() {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField("agent_did"));
    }
    if record.timestamp_epoch_seconds == 0 {
        return Err(DataLayerPgRepositoryBridgeError::EmptyField(
            "timestamp_epoch_seconds",
        ));
    }
    let expected_hour_bucket = record.timestamp_epoch_seconds
        - (record.timestamp_epoch_seconds % DATA_LAYER_M7_HOURLY_BUCKET_SECONDS);
    let expected_day_bucket = record.timestamp_epoch_seconds
        - (record.timestamp_epoch_seconds % DATA_LAYER_M7_DAILY_BUCKET_SECONDS);
    if record.bucket_hour_epoch_seconds != expected_hour_bucket {
        return Err(
            DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                bucket_window_seconds: DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
            },
        );
    }
    if record.bucket_day_epoch_seconds != expected_day_bucket {
        return Err(
            DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                bucket_window_seconds: DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
            },
        );
    }
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::InsertTelemetryPoint,
        sql: format!(
            "INSERT INTO {} (owner_did, agent_did, observed_at, bucket_hour_epoch_seconds, bucket_day_epoch_seconds, message_count, bytes_stored, query_count, embedding_count, embedding_anomaly_count, ingress_latency_ms_p95, egress_latency_ms_p95, active_sessions, sequence) VALUES ($1, $2, to_timestamp($3), $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14);",
            config.hypertable_name
        ),
        bind_markers: vec!["owner_did", "agent_did", "timestamp_epoch_seconds", "bucket_hour_epoch_seconds", "bucket_day_epoch_seconds", "message_count", "bytes_stored", "query_count", "embedding_count", "embedding_anomaly_count", "ingress_latency_ms_p95", "egress_latency_ms_p95", "active_sessions", "sequence"],
        session: build_requester_session(requester_did)?,
    })
}

pub fn data_layer_pg_project_m7_timescale_owner_rollup_query_operation(
    request: DataLayerPgM7TimescaleOwnerRollupRequest,
    config: DataLayerPgM7TimescaleConfig,
) -> Result<DataLayerPgSqlOperation, DataLayerPgRepositoryBridgeError> {
    validate_timescale_config(&config)?;
    validate_owner_did(request.query.owner_did.as_str())?;
    validate_owner_did(request.query.requester_owner_did.as_str())?;
    let interval_marker = match request.bucket_window_seconds {
        DATA_LAYER_M7_HOURLY_BUCKET_SECONDS => "1 hour",
        DATA_LAYER_M7_DAILY_BUCKET_SECONDS => "1 day",
        other => {
            return Err(
                DataLayerPgRepositoryBridgeError::InvalidTimescaleBucketWindow {
                    reason_code: DATA_LAYER_PG_TIMESCALE_INVALID_BUCKET_WINDOW_REASON_CODE,
                    bucket_window_seconds: other,
                },
            );
        }
    };
    let limit = request
        .limit
        .unwrap_or(DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT);
    if limit == 0 || limit > DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT {
        return Err(DataLayerPgRepositoryBridgeError::InvalidSearchLimit {
            requested: limit as u32,
            max_allowed: DATA_LAYER_PG_MAX_TIMESCALE_QUERY_LIMIT as u32,
        });
    }
    Ok(DataLayerPgSqlOperation {
        kind: DataLayerPgOperationKind::QueryTelemetryOwnerRollup,
        sql: format!(
            "SELECT time_bucket(INTERVAL '{}', observed_at) AS bucket_start, SUM(message_count) AS message_count_total, SUM(bytes_stored) AS bytes_stored_total, SUM(query_count) AS query_count_total, SUM(embedding_count) AS embedding_count_total FROM {} WHERE owner_did = $1 GROUP BY bucket_start ORDER BY bucket_start DESC LIMIT $2;",
            interval_marker, config.hypertable_name
        ),
        bind_markers: vec!["owner_did", "limit"],
        session: build_requester_session(request.requester_did.as_str())?,
    })
}
