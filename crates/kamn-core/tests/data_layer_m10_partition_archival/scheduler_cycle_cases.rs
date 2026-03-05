use super::*;

const OWNER_SCHEDULER_DEFERRED: &str = "kamn:did:owner:phase6-scheduler-deferred";
const OWNER_SCHEDULER_PREFLIGHT: &str = "kamn:did:owner:phase6-scheduler-preflight";
const OWNER_SCHEDULER_TRIGGERED: &str = "kamn:did:owner:phase6-scheduler-triggered";
const MESSAGE_DEFER: &str = "defer-message";

pub(super) fn run_spec_c23_phase6_scheduler_trigger_decision_orders_due_threshold_interval_and_deferred(
) {
    let due_threshold_triggered = data_layer_m10_evaluate_phase6_scheduler_trigger(
        phase6_scheduler_policy(2, 300),
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count: 2,
            last_tick_epoch_seconds: Some(1_700_000_900),
            now_epoch_seconds: 1_700_001_000,
        },
    )
    .expect("due-threshold trigger evaluation should succeed");
    assert!(matches!(
        due_threshold_triggered,
        DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
            ..
        }
    ));

    let interval_triggered = data_layer_m10_evaluate_phase6_scheduler_trigger(
        phase6_scheduler_policy(5, 120),
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count: 1,
            last_tick_epoch_seconds: Some(1_700_000_700),
            now_epoch_seconds: 1_700_001_000,
        },
    )
    .expect("interval trigger evaluation should succeed");
    assert!(matches!(
        interval_triggered,
        DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_INTERVAL_ELAPSED_REASON_CODE,
            ..
        }
    ));

    let deferred = data_layer_m10_evaluate_phase6_scheduler_trigger(
        phase6_scheduler_policy(5, 600),
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count: 1,
            last_tick_epoch_seconds: Some(1_700_000_800),
            now_epoch_seconds: 1_700_001_000,
        },
    )
    .expect("deferred trigger evaluation should succeed");
    assert!(matches!(
        deferred,
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
            ..
        }
    ));
}

pub(super) fn run_spec_c24_phase6_scheduler_cycle_deferred_path_returns_no_execution_side_effects()
{
    let owner_did = OWNER_SCHEDULER_DEFERRED;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202601, false))
        .expect("partition should register");

    let mut recent_message = m8_message_input(owner_did, MESSAGE_DEFER, 1_699_999_990);
    recent_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(recent_message)
        .expect("message should register");

    let cycle_report = data_layer_m10_execute_phase6_scheduler_cycle(
        &mut m8_registry,
        &mut m10_registry,
        DataLayerM10Phase6SchedulerCycleRequest {
            scheduler_policy: phase6_scheduler_policy(2, 600),
            last_tick_epoch_seconds: Some(1_699_999_700),
            budget: phase6_budget(2, 2, 1, 1),
            execution_request: phase6_request(
                owner_did,
                BTreeMap::from([(202601, vec![MESSAGE_DEFER.to_owned()])]),
            ),
        },
    )
    .expect("deferred cycle should succeed");
    assert_eq!(
        cycle_report.reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE
    );
    assert!(matches!(
        cycle_report.trigger_decision,
        DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
            ..
        }
    ));
    assert!(cycle_report.execution_report.is_none());
    assert!(cycle_report.budget_report.is_none());

    let message = m8_registry
        .message_for_owner(owner_did, MESSAGE_DEFER)
        .expect("message should exist");
    assert!(message.shredded_at_epoch_seconds.is_none());
}

pub(super) fn run_spec_c25_phase6_scheduler_cycle_preflight_budget_overflow_fails_closed_before_execution(
) {
    let owner_did = OWNER_SCHEDULER_PREFLIGHT;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("preflight-a", 1_699_700_000_u64),
        ("preflight-b", 1_699_700_100_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let preflight_error = data_layer_m10_execute_phase6_scheduler_cycle(
        &mut m8_registry,
        &mut m10_registry,
        DataLayerM10Phase6SchedulerCycleRequest {
            scheduler_policy: phase6_scheduler_policy(1, 600),
            last_tick_epoch_seconds: Some(1_699_999_900),
            budget: phase6_budget(1, 3, 2, 2),
            execution_request: phase6_request(
                owner_did,
                BTreeMap::from([(
                    202401,
                    vec!["preflight-a".to_owned(), "preflight-b".to_owned()],
                )]),
            ),
        },
    );
    assert!(matches!(
        preflight_error,
        Err(
            DataLayerM10PartitionLifecycleError::Phase6SchedulerBudgetPreflightExceeded {
                reason_code:
                    DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE,
                ..
            }
        )
    ));

    let message = m8_registry
        .message_for_owner(owner_did, "preflight-a")
        .expect("message should exist");
    assert!(message.shredded_at_epoch_seconds.is_none());
}

pub(super) fn run_spec_c26_phase6_scheduler_cycle_triggered_executes_within_budget_evidence() {
    let owner_did = OWNER_SCHEDULER_TRIGGERED;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("trigger-a", 1_699_700_000_u64),
        ("trigger-b", 1_699_700_100_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let cycle_report = data_layer_m10_execute_phase6_scheduler_cycle(
        &mut m8_registry,
        &mut m10_registry,
        DataLayerM10Phase6SchedulerCycleRequest {
            scheduler_policy: phase6_scheduler_policy(1, 600),
            last_tick_epoch_seconds: Some(1_699_999_900),
            budget: phase6_budget(2, 2, 1, 1),
            execution_request: phase6_request(
                owner_did,
                BTreeMap::from([(202401, vec!["trigger-b".to_owned(), "trigger-a".to_owned()])]),
            ),
        },
    )
    .expect("triggered cycle should execute");
    assert_eq!(
        cycle_report.reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE
    );
    assert!(matches!(
        cycle_report.trigger_decision,
        DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
            ..
        }
    ));
    let execution_report = cycle_report
        .execution_report
        .expect("execution report should be populated");
    assert_eq!(execution_report.due_candidate_count, 2);
    assert_eq!(execution_report.archived_entries.len(), 1);
    let budget_report = cycle_report
        .budget_report
        .expect("budget report should be populated");
    assert_eq!(
        budget_report.decision,
        DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget
    );
    assert_eq!(
        budget_report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE
    );
}

pub(super) fn run_spec_c27_phase6_scheduler_policy_and_signal_validation_fail_closed() {
    let invalid_policy = data_layer_m10_evaluate_phase6_scheduler_trigger(
        phase6_scheduler_policy(0, 60),
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count: 0,
            last_tick_epoch_seconds: Some(1_700_000_900),
            now_epoch_seconds: 1_700_001_000,
        },
    );
    assert!(matches!(
        invalid_policy,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerPolicy {
                field: "due_candidate_trigger_threshold",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_POLICY_INVALID_REASON_CODE,
            }
        )
    ));

    let invalid_signal = data_layer_m10_evaluate_phase6_scheduler_trigger(
        phase6_scheduler_policy(1, 60),
        DataLayerM10Phase6SchedulerSignal {
            due_candidate_count: 0,
            last_tick_epoch_seconds: Some(1_700_001_001),
            now_epoch_seconds: 1_700_001_000,
        },
    );
    assert!(matches!(
        invalid_signal,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                field: "last_tick_epoch_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
            }
        )
    ));
}
