//! Cross-store replay consistency checker and divergence taxonomy contracts.

use crate::{
    ChannelSnapshot, MessageLifecycleSnapshot, RuntimeSnapshot, TaskOperationSnapshot,
    CHANNEL_SNAPSHOT_SCHEMA_VERSION, MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
    TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION,
};
use tracing::debug;

const CROSS_STORE_REPLAY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.cross-store-replay-consistency-reason-taxonomy.v1";
const CROSS_STORE_REPLAY_REASON_CODES_CSV: &str = "none,cross_store_replay_divergence_all_snapshots_missing,cross_store_replay_divergence_runtime_snapshot_missing,cross_store_replay_divergence_channel_snapshot_missing,cross_store_replay_divergence_message_snapshot_missing,cross_store_replay_divergence_task_snapshot_missing,cross_store_replay_divergence_channel_schema_version_mismatch,cross_store_replay_divergence_message_schema_version_mismatch,cross_store_replay_divergence_task_schema_version_mismatch,cross_store_replay_divergence_runtime_cursor_state_version_mismatch,cross_store_replay_divergence_aggregate_records_missing_for_advanced_runtime_state,cross_store_replay_divergence_aggregate_records_exceed_runtime_cursor";

/// Deterministic status emitted by the cross-store replay consistency checker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossStoreReplayConsistencyStatus {
    /// All required cross-store checks passed.
    Consistent,
    /// One or more cross-store checks failed.
    Divergent,
}

/// Deterministic divergence classes for replay consistency failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossStoreReplayDivergenceClass {
    /// No divergence was detected.
    Consistent,
    /// Snapshot presence drift across required stores.
    PresenceDrift,
    /// Snapshot schema-version drift across required stores.
    SchemaDrift,
    /// Runtime snapshot continuity drift for state_version/cursor contracts.
    RuntimeContinuityDrift,
    /// Cross-store cardinality drift between runtime cursor and domain records.
    CardinalityDrift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SnapshotProjection {
    runtime_state_version: Option<u64>,
    runtime_cursor: Option<u64>,
    channel_schema_version: Option<u16>,
    message_schema_version: Option<u16>,
    task_schema_version: Option<u16>,
    channel_record_count: Option<usize>,
    message_record_count: Option<usize>,
    task_record_count: Option<usize>,
}

impl SnapshotProjection {
    fn from_snapshots(
        runtime_snapshot: Option<&RuntimeSnapshot>,
        channel_snapshot: Option<&ChannelSnapshot>,
        message_snapshot: Option<&MessageLifecycleSnapshot>,
        task_snapshot: Option<&TaskOperationSnapshot>,
    ) -> Self {
        Self {
            runtime_state_version: runtime_snapshot.map(RuntimeSnapshot::state_version),
            runtime_cursor: runtime_snapshot.map(RuntimeSnapshot::cursor),
            channel_schema_version: channel_snapshot.map(|snapshot| snapshot.schema_version),
            message_schema_version: message_snapshot.map(|snapshot| snapshot.schema_version),
            task_schema_version: task_snapshot.map(|snapshot| snapshot.schema_version),
            channel_record_count: channel_snapshot.map(|snapshot| snapshot.records.len()),
            message_record_count: message_snapshot.map(|snapshot| snapshot.records.len()),
            task_record_count: task_snapshot.map(|snapshot| snapshot.tasks.len()),
        }
    }

    fn aggregate_record_count(&self) -> Option<usize> {
        match (
            self.channel_record_count,
            self.message_record_count,
            self.task_record_count,
        ) {
            (Some(channel), Some(message), Some(task)) => Some(channel + message + task),
            _ => None,
        }
    }

    fn consistency_fingerprint(&self, reason_code: &str) -> String {
        format!(
            "runtime:{}:{}|channel:{}:{}|message:{}:{}|task:{}:{}|aggregate:{}|reason:{}",
            render_opt(self.runtime_state_version),
            render_opt(self.runtime_cursor),
            render_opt(self.channel_schema_version),
            render_opt(self.channel_record_count),
            render_opt(self.message_schema_version),
            render_opt(self.message_record_count),
            render_opt(self.task_schema_version),
            render_opt(self.task_record_count),
            render_opt(self.aggregate_record_count()),
            reason_code,
        )
    }
}

/// Deterministic report emitted by replay consistency evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossStoreReplayConsistencyReport {
    status: CrossStoreReplayConsistencyStatus,
    reason_code: String,
    divergence_class: CrossStoreReplayDivergenceClass,
    source_marker: &'static str,
    runtime_state_version: Option<u64>,
    runtime_cursor: Option<u64>,
    channel_schema_version: Option<u16>,
    message_schema_version: Option<u16>,
    task_schema_version: Option<u16>,
    channel_record_count: Option<usize>,
    message_record_count: Option<usize>,
    task_record_count: Option<usize>,
    aggregate_record_count: Option<usize>,
    consistency_fingerprint: String,
}

impl CrossStoreReplayConsistencyReport {
    fn new(
        status: CrossStoreReplayConsistencyStatus,
        reason_code: &str,
        divergence_class: CrossStoreReplayDivergenceClass,
        projection: SnapshotProjection,
    ) -> Self {
        Self {
            status,
            reason_code: reason_code.to_owned(),
            divergence_class,
            source_marker: "cross_store_replay_consistency_checker",
            runtime_state_version: projection.runtime_state_version,
            runtime_cursor: projection.runtime_cursor,
            channel_schema_version: projection.channel_schema_version,
            message_schema_version: projection.message_schema_version,
            task_schema_version: projection.task_schema_version,
            channel_record_count: projection.channel_record_count,
            message_record_count: projection.message_record_count,
            task_record_count: projection.task_record_count,
            aggregate_record_count: projection.aggregate_record_count(),
            consistency_fingerprint: projection.consistency_fingerprint(reason_code),
        }
    }

    /// Returns deterministic consistency status.
    pub fn status(&self) -> CrossStoreReplayConsistencyStatus {
        self.status
    }

    /// Returns deterministic reason code.
    pub fn reason_code(&self) -> &str {
        self.reason_code.as_str()
    }

    /// Returns deterministic divergence class.
    pub fn divergence_class(&self) -> CrossStoreReplayDivergenceClass {
        self.divergence_class
    }

    /// Returns deterministic source marker.
    pub fn source_marker(&self) -> &'static str {
        self.source_marker
    }

    /// Returns deterministic reason taxonomy version marker.
    pub fn reason_taxonomy_version(&self) -> &'static str {
        cross_store_replay_reason_taxonomy_version()
    }

    /// Returns deterministic runtime state version projection.
    pub fn runtime_state_version(&self) -> Option<u64> {
        self.runtime_state_version
    }

    /// Returns deterministic runtime cursor projection.
    pub fn runtime_cursor(&self) -> Option<u64> {
        self.runtime_cursor
    }

    /// Returns deterministic channel schema version projection.
    pub fn channel_schema_version(&self) -> Option<u16> {
        self.channel_schema_version
    }

    /// Returns deterministic message schema version projection.
    pub fn message_schema_version(&self) -> Option<u16> {
        self.message_schema_version
    }

    /// Returns deterministic task schema version projection.
    pub fn task_schema_version(&self) -> Option<u16> {
        self.task_schema_version
    }

    /// Returns deterministic channel record count projection.
    pub fn channel_record_count(&self) -> Option<usize> {
        self.channel_record_count
    }

    /// Returns deterministic message record count projection.
    pub fn message_record_count(&self) -> Option<usize> {
        self.message_record_count
    }

    /// Returns deterministic task record count projection.
    pub fn task_record_count(&self) -> Option<usize> {
        self.task_record_count
    }

    /// Returns deterministic aggregate record count projection.
    pub fn aggregate_record_count(&self) -> Option<usize> {
        self.aggregate_record_count
    }

    /// Returns deterministic replay-consistency fingerprint.
    pub fn consistency_fingerprint(&self) -> &str {
        self.consistency_fingerprint.as_str()
    }
}

/// Returns deterministic cross-store replay reason taxonomy marker.
pub fn cross_store_replay_reason_taxonomy_version() -> &'static str {
    CROSS_STORE_REPLAY_REASON_TAXONOMY_VERSION
}

/// Returns deterministic ordered reason-code CSV for replay consistency checker policy gates.
pub fn cross_store_replay_reason_codes_csv() -> &'static str {
    CROSS_STORE_REPLAY_REASON_CODES_CSV
}

/// Evaluates runtime/channel/message/task snapshot parity and emits deterministic divergence output.
pub fn evaluate_cross_store_replay_consistency(
    runtime_snapshot: Option<RuntimeSnapshot>,
    channel_snapshot: Option<ChannelSnapshot>,
    message_snapshot: Option<MessageLifecycleSnapshot>,
    task_snapshot: Option<TaskOperationSnapshot>,
) -> CrossStoreReplayConsistencyReport {
    let projection = SnapshotProjection::from_snapshots(
        runtime_snapshot.as_ref(),
        channel_snapshot.as_ref(),
        message_snapshot.as_ref(),
        task_snapshot.as_ref(),
    );

    if runtime_snapshot.is_none()
        && channel_snapshot.is_none()
        && message_snapshot.is_none()
        && task_snapshot.is_none()
    {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_all_snapshots_missing",
            CrossStoreReplayDivergenceClass::PresenceDrift,
            projection,
        );
    }

    if runtime_snapshot.is_none() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_runtime_snapshot_missing",
            CrossStoreReplayDivergenceClass::PresenceDrift,
            projection,
        );
    }
    if channel_snapshot.is_none() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_channel_snapshot_missing",
            CrossStoreReplayDivergenceClass::PresenceDrift,
            projection,
        );
    }
    if message_snapshot.is_none() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_message_snapshot_missing",
            CrossStoreReplayDivergenceClass::PresenceDrift,
            projection,
        );
    }
    if task_snapshot.is_none() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_task_snapshot_missing",
            CrossStoreReplayDivergenceClass::PresenceDrift,
            projection,
        );
    }

    let (runtime_snapshot, channel_snapshot, message_snapshot, task_snapshot) = match (
        runtime_snapshot,
        channel_snapshot,
        message_snapshot,
        task_snapshot,
    ) {
        (Some(runtime), Some(channel), Some(message), Some(task)) => {
            (runtime, channel, message, task)
        }
        _ => unreachable!("snapshot presence checks completed before tuple extraction"),
    };

    if channel_snapshot.schema_version != CHANNEL_SNAPSHOT_SCHEMA_VERSION {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_channel_schema_version_mismatch",
            CrossStoreReplayDivergenceClass::SchemaDrift,
            projection,
        );
    }
    if message_snapshot.schema_version != MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_message_schema_version_mismatch",
            CrossStoreReplayDivergenceClass::SchemaDrift,
            projection,
        );
    }
    if task_snapshot.schema_version != TASK_OPERATION_SNAPSHOT_SCHEMA_VERSION {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_task_schema_version_mismatch",
            CrossStoreReplayDivergenceClass::SchemaDrift,
            projection,
        );
    }

    if runtime_snapshot.cursor() < runtime_snapshot.state_version() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_runtime_cursor_state_version_mismatch",
            CrossStoreReplayDivergenceClass::RuntimeContinuityDrift,
            projection,
        );
    }

    let aggregate_record_count = projection.aggregate_record_count().unwrap_or(0);
    if runtime_snapshot.state_version() > 1 && aggregate_record_count == 0 {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_aggregate_records_missing_for_advanced_runtime_state",
            CrossStoreReplayDivergenceClass::CardinalityDrift,
            projection,
        );
    }

    let aggregate_record_count_u64 = u64::try_from(aggregate_record_count).unwrap_or(u64::MAX);
    if aggregate_record_count_u64 > runtime_snapshot.cursor() {
        return build_and_trace_report(
            CrossStoreReplayConsistencyStatus::Divergent,
            "cross_store_replay_divergence_aggregate_records_exceed_runtime_cursor",
            CrossStoreReplayDivergenceClass::CardinalityDrift,
            projection,
        );
    }

    build_and_trace_report(
        CrossStoreReplayConsistencyStatus::Consistent,
        "none",
        CrossStoreReplayDivergenceClass::Consistent,
        projection,
    )
}

fn build_and_trace_report(
    status: CrossStoreReplayConsistencyStatus,
    reason_code: &str,
    divergence_class: CrossStoreReplayDivergenceClass,
    projection: SnapshotProjection,
) -> CrossStoreReplayConsistencyReport {
    let report =
        CrossStoreReplayConsistencyReport::new(status, reason_code, divergence_class, projection);
    debug!(
        reason_code = report.reason_code(),
        divergence_class = ?report.divergence_class(),
        status = ?report.status(),
        reason_taxonomy_version = report.reason_taxonomy_version(),
        consistency_fingerprint = report.consistency_fingerprint(),
        "evaluated cross-store replay consistency"
    );
    report
}

fn render_opt<T: ToString>(value: Option<T>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "none".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
        render_opt, CROSS_STORE_REPLAY_REASON_TAXONOMY_VERSION,
    };

    #[test]
    fn unit_cross_store_replay_reason_taxonomy_marker_is_stable() {
        assert_eq!(
            cross_store_replay_reason_taxonomy_version(),
            CROSS_STORE_REPLAY_REASON_TAXONOMY_VERSION
        );
    }

    #[test]
    fn unit_cross_store_replay_reason_codes_include_expected_presence_drift_reason() {
        assert!(cross_store_replay_reason_codes_csv()
            .contains("cross_store_replay_divergence_runtime_snapshot_missing"));
    }

    #[test]
    fn unit_cross_store_replay_render_opt_outputs_none_for_missing_values() {
        assert_eq!(render_opt::<u64>(None), "none");
    }
}
