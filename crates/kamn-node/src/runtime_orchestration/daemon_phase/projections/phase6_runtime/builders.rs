use super::super::super::super::*;

pub(super) fn build_phase6_runtime(
    has_shutdown_signal: bool,
) -> Result<kamn_core::DataLayerM10Phase6SchedulerRuntime, ConfigError> {
    kamn_core::DataLayerM10Phase6SchedulerRuntime::new(
        scheduler_policy(has_shutdown_signal),
        execution_tick_budget(),
    )
    .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}

fn scheduler_policy(has_shutdown_signal: bool) -> kamn_core::DataLayerM10Phase6SchedulerPolicy {
    if has_shutdown_signal {
        return kamn_core::DataLayerM10Phase6SchedulerPolicy {
            due_candidate_trigger_threshold: 2,
            max_tick_interval_seconds: 2_000_000_000,
        };
    }
    kamn_core::DataLayerM10Phase6SchedulerPolicy {
        due_candidate_trigger_threshold: 1,
        max_tick_interval_seconds: 60,
    }
}

fn execution_tick_budget() -> kamn_core::DataLayerM10Phase6ExecutionTickBudget {
    kamn_core::DataLayerM10Phase6ExecutionTickBudget {
        max_due_candidates: 2,
        max_shredded_messages: 2,
        max_projection_reports: 1,
        max_archived_entries: 1,
    }
}

pub(super) fn build_phase6_request(
    owner_did: &str,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
    partition_message_ids_by_month: std::collections::BTreeMap<u32, Vec<String>>,
) -> kamn_core::DataLayerM10Phase6ExecutionTickRequest {
    kamn_core::DataLayerM10Phase6ExecutionTickRequest {
        requester_owner_did: owner_did.to_owned(),
        owner_did: owner_did.to_owned(),
        now_epoch_seconds: phase6_now_epoch_seconds(tick_interval_ms, has_shutdown_signal),
        shredded_at_epoch_seconds: 1_700_000_300,
        now_month_id: 202602,
        active_retention_months: 2,
        object_storage_prefix: "s3://kamn-archive/messages".to_owned(),
        partition_message_ids_by_month,
    }
}

fn phase6_now_epoch_seconds(tick_interval_ms: u64, has_shutdown_signal: bool) -> u64 {
    if has_shutdown_signal {
        return 1_700_000_010_u64;
    }
    1_700_000_000_u64 + tick_interval_ms.saturating_add(95)
}
