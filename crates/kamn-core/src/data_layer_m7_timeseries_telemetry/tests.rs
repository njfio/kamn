#[cfg(test)]
mod contracts {
    use super::super::{
        daily_bucket, hourly_bucket, project_m7_observability_projection, validate_kamn_did,
        DataLayerM7TelemetryPointRecord, DataLayerM7TimeseriesError,
        DATA_LAYER_M7_DAILY_BUCKET_SECONDS, DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
    };

    fn telemetry_point(
        message_count: u64,
        query_count: u64,
        embedding_count: u64,
        embedding_anomaly_count: u64,
        active_sessions: u32,
    ) -> DataLayerM7TelemetryPointRecord {
        DataLayerM7TelemetryPointRecord {
            owner_did: "kamn:did:owner:alpha".to_owned(),
            agent_did: "kamn:did:agent:processor".to_owned(),
            timestamp_epoch_seconds: 1_735_689_600,
            bucket_hour_epoch_seconds: 1_735_689_600,
            bucket_day_epoch_seconds: 1_735_660_800,
            message_count,
            bytes_stored: 128,
            query_count,
            embedding_count,
            embedding_anomaly_count,
            ingress_latency_ms_p95: 12,
            egress_latency_ms_p95: 18,
            active_sessions,
            sequence: 1,
        }
    }

    #[test]
    fn unit_observability_throughput_uses_minimum_floor_and_session_boost() {
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(0, 0, 0, 0, 0)).throughput_tps,
            1
        );
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(2, 3, 5, 0, 4)).throughput_tps,
            4_010
        );
    }

    #[test]
    fn unit_observability_error_rate_and_availability_are_bounded() {
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(0, 0, 0, 9, 0)).error_rate_pct,
            0.0
        );
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(0, 0, 10, 25, 0)).error_rate_pct,
            100.0
        );
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(0, 0, 0, 0, 0)).availability_pct,
            0.0
        );
        assert_eq!(
            project_m7_observability_projection(&telemetry_point(0, 0, 0, 0, 1)).availability_pct,
            100.0
        );
    }

    #[test]
    fn unit_validate_kamn_did_and_bucket_helpers_enforce_contracts() {
        assert_eq!(
            validate_kamn_did("did:example:alice"),
            Err(DataLayerM7TimeseriesError::InvalidDid(
                "did:example:alice".to_owned()
            ))
        );
        assert!(validate_kamn_did("kamn:did:owner:alpha").is_ok());

        let timestamp = 1_735_700_123;
        assert_eq!(
            hourly_bucket(timestamp) % DATA_LAYER_M7_HOURLY_BUCKET_SECONDS,
            0
        );
        assert_eq!(
            daily_bucket(timestamp) % DATA_LAYER_M7_DAILY_BUCKET_SECONDS,
            0
        );
    }
}
