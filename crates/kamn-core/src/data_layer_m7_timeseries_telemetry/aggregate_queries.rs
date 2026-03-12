use super::{
    authorize_owner_scope, validate_kamn_did, DataLayerM7AgentDailyAggregate,
    DataLayerM7AgentHourlyAggregate, DataLayerM7NetworkHourlyAggregate,
    DataLayerM7TelemetryPointRecord, DataLayerM7TelemetryScopeQuery, DataLayerM7TimeseriesError,
    DATA_LAYER_M7_AGGREGATE_REASON_CODE, DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
};
use std::collections::{BTreeMap, BTreeSet};

type DataLayerM7NetworkHourlyAccumulator = (u64, u64, u64, u64, u64, BTreeSet<String>);
type DataLayerM7AgentAccumulator = (u64, u64, u64, u64, u64);

pub(crate) fn aggregate_agent_hourly(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    query: &DataLayerM7TelemetryScopeQuery,
) -> Result<Vec<DataLayerM7AgentHourlyAggregate>, DataLayerM7TimeseriesError> {
    validate_agent_scope(query)?;
    Ok(group_agent_points(owner_points, query, |point| point.bucket_hour_epoch_seconds)
        .into_iter()
        .map(|(bucket_hour_epoch_seconds, totals)| build_agent_hourly(query, bucket_hour_epoch_seconds, totals))
        .collect())
}

pub(crate) fn aggregate_agent_daily(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    query: &DataLayerM7TelemetryScopeQuery,
) -> Result<Vec<DataLayerM7AgentDailyAggregate>, DataLayerM7TimeseriesError> {
    validate_agent_scope(query)?;
    Ok(group_agent_points(owner_points, query, |point| point.bucket_day_epoch_seconds)
        .into_iter()
        .map(|(bucket_day_epoch_seconds, totals)| build_agent_daily(query, bucket_day_epoch_seconds, totals))
        .collect())
}

pub(crate) fn aggregate_network_hourly(
    points_by_owner: &BTreeMap<String, Vec<DataLayerM7TelemetryPointRecord>>,
) -> Vec<DataLayerM7NetworkHourlyAggregate> {
    group_network_points(points_by_owner)
        .into_iter()
        .map(|(bucket_hour_epoch_seconds, totals)| build_network_hourly(bucket_hour_epoch_seconds, totals))
        .collect()
}

fn group_agent_points(
    owner_points: &[DataLayerM7TelemetryPointRecord],
    query: &DataLayerM7TelemetryScopeQuery,
    bucket_of: fn(&DataLayerM7TelemetryPointRecord) -> u64,
) -> BTreeMap<u64, DataLayerM7AgentAccumulator> {
    let mut grouped = BTreeMap::new();
    for point in owner_points.iter().filter(|point| point.agent_did == query.agent_did) {
        accumulate_agent_totals(grouped.entry(bucket_of(point)).or_insert((0, 0, 0, 0, 0)), point);
    }
    grouped
}

fn validate_agent_scope(
    query: &DataLayerM7TelemetryScopeQuery,
) -> Result<(), DataLayerM7TimeseriesError> {
    authorize_owner_scope(
        query.requester_owner_did.as_str(),
        query.owner_did.as_str(),
        DATA_LAYER_M7_OWNER_SCOPE_DENIED_REASON_CODE,
    )?;
    validate_kamn_did(query.agent_did.as_str())
}

fn accumulate_agent_totals(
    entry: &mut DataLayerM7AgentAccumulator,
    point: &DataLayerM7TelemetryPointRecord,
) {
    entry.0 += point.message_count;
    entry.1 += point.bytes_stored;
    entry.2 += point.query_count;
    entry.3 += point.embedding_count;
    entry.4 += point.embedding_anomaly_count;
}

fn build_agent_hourly(
    query: &DataLayerM7TelemetryScopeQuery,
    bucket_hour_epoch_seconds: u64,
    totals: DataLayerM7AgentAccumulator,
) -> DataLayerM7AgentHourlyAggregate {
    DataLayerM7AgentHourlyAggregate {
        owner_did: query.owner_did.clone(),
        agent_did: query.agent_did.clone(),
        bucket_hour_epoch_seconds,
        message_count_total: totals.0,
        bytes_stored_total: totals.1,
        query_count_total: totals.2,
        embedding_count_total: totals.3,
        embedding_anomaly_count_total: totals.4,
        reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
    }
}

fn build_agent_daily(
    query: &DataLayerM7TelemetryScopeQuery,
    bucket_day_epoch_seconds: u64,
    totals: DataLayerM7AgentAccumulator,
) -> DataLayerM7AgentDailyAggregate {
    DataLayerM7AgentDailyAggregate {
        owner_did: query.owner_did.clone(),
        agent_did: query.agent_did.clone(),
        bucket_day_epoch_seconds,
        message_count_total: totals.0,
        bytes_stored_total: totals.1,
        query_count_total: totals.2,
        embedding_count_total: totals.3,
        embedding_anomaly_count_total: totals.4,
        reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
    }
}

fn group_network_points(
    points_by_owner: &BTreeMap<String, Vec<DataLayerM7TelemetryPointRecord>>,
) -> BTreeMap<u64, DataLayerM7NetworkHourlyAccumulator> {
    let mut grouped = BTreeMap::new();
    for owner_points in points_by_owner.values() {
        for point in owner_points {
            accumulate_network_totals(
                grouped
                    .entry(point.bucket_hour_epoch_seconds)
                    .or_insert((0, 0, 0, 0, 0, BTreeSet::new())),
                point,
            );
        }
    }
    grouped
}

fn accumulate_network_totals(
    entry: &mut DataLayerM7NetworkHourlyAccumulator,
    point: &DataLayerM7TelemetryPointRecord,
) {
    entry.0 += point.message_count;
    entry.1 += point.bytes_stored;
    entry.2 += point.query_count;
    entry.3 += point.embedding_count;
    entry.4 += point.embedding_anomaly_count;
    entry.5.insert(point.agent_did.clone());
}

fn build_network_hourly(
    bucket_hour_epoch_seconds: u64,
    totals: DataLayerM7NetworkHourlyAccumulator,
) -> DataLayerM7NetworkHourlyAggregate {
    DataLayerM7NetworkHourlyAggregate {
        bucket_hour_epoch_seconds,
        message_count_total: totals.0,
        bytes_stored_total: totals.1,
        query_count_total: totals.2,
        embedding_count_total: totals.3,
        embedding_anomaly_count_total: totals.4,
        active_agent_count: totals.5.len() as u64,
        reason_code: DATA_LAYER_M7_AGGREGATE_REASON_CODE,
    }
}
