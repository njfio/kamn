use super::*;

pub(super) fn run_spec_c28_phase6_scheduler_runtime_initializes_zeroed_state_and_checkpoint() {
    let runtime = DataLayerM10Phase6SchedulerRuntime::new(
        phase6_scheduler_policy(1, 600),
        phase6_budget(1, 1, 1, 1),
    )
    .expect("runtime should initialize");
    let state = runtime.state();
    assert_eq!(state.last_successful_tick_epoch_seconds, None);
    assert_eq!(state.last_observed_now_epoch_seconds, None);
    assert_eq!(state.total_cycles, 0);
    assert_eq!(state.executed_cycles, 0);
    assert_eq!(state.deferred_cycles, 0);
    assert_eq!(state.fail_closed_cycles, 0);
    assert_eq!(
        state.last_reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_RUNTIME_INITIALIZED_REASON_CODE
    );
}

pub(super) fn run_spec_c29_phase6_scheduler_runtime_deferred_cycle_preserves_success_checkpoint() {
    let owner_did = "kamn:did:owner:phase6-runtime-deferred";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202601, false))
        .expect("partition should register");

    let mut recent_message = m8_message_input(owner_did, "runtime-defer-message", 1_699_999_990);
    recent_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(recent_message)
        .expect("message should register");

    let mut runtime = DataLayerM10Phase6SchedulerRuntime::new(
        phase6_scheduler_policy(2, 2_000_000_000),
        phase6_budget(2, 2, 1, 1),
    )
    .expect("runtime should initialize");
    let cycle_report = runtime
        .run_cycle(
            &mut m8_registry,
            &mut m10_registry,
            phase6_request(
                owner_did,
                BTreeMap::from([(202601, vec!["runtime-defer-message".to_owned()])]),
            ),
        )
        .expect("deferred runtime cycle should succeed");
    assert_eq!(
        cycle_report.reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE
    );
    let state = runtime.state();
    assert_eq!(state.last_successful_tick_epoch_seconds, None);
    assert_eq!(state.last_observed_now_epoch_seconds, Some(1_700_000_000));
    assert_eq!(state.total_cycles, 1);
    assert_eq!(state.executed_cycles, 0);
    assert_eq!(state.deferred_cycles, 1);
    assert_eq!(state.fail_closed_cycles, 0);
    assert_eq!(
        state.last_reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE
    );
}

pub(super) fn run_spec_c30_phase6_scheduler_runtime_applied_cycle_updates_success_checkpoint_and_counters(
) {
    let owner_did = "kamn:did:owner:phase6-runtime-applied";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("runtime-apply-a", 1_699_700_000_u64),
        ("runtime-apply-b", 1_699_700_100_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let mut runtime = DataLayerM10Phase6SchedulerRuntime::new(
        phase6_scheduler_policy(1, 600),
        phase6_budget(2, 2, 1, 1),
    )
    .expect("runtime should initialize");
    let cycle_report = runtime
        .run_cycle(
            &mut m8_registry,
            &mut m10_registry,
            phase6_request(
                owner_did,
                BTreeMap::from([(
                    202401,
                    vec!["runtime-apply-b".to_owned(), "runtime-apply-a".to_owned()],
                )]),
            ),
        )
        .expect("applied runtime cycle should succeed");
    assert_eq!(
        cycle_report.reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE
    );
    let state = runtime.state();
    assert_eq!(
        state.last_successful_tick_epoch_seconds,
        Some(1_700_000_000)
    );
    assert_eq!(state.last_observed_now_epoch_seconds, Some(1_700_000_000));
    assert_eq!(state.total_cycles, 1);
    assert_eq!(state.executed_cycles, 1);
    assert_eq!(state.deferred_cycles, 0);
    assert_eq!(state.fail_closed_cycles, 0);
    assert_eq!(
        state.last_reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE
    );
}

pub(super) fn run_spec_c31_phase6_scheduler_runtime_preflight_fail_closed_increments_fail_counter_without_checkpoint_advance(
) {
    let owner_did = "kamn:did:owner:phase6-runtime-preflight";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("runtime-preflight-a", 1_699_700_000_u64),
        ("runtime-preflight-b", 1_699_700_100_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let mut runtime = DataLayerM10Phase6SchedulerRuntime::new(
        phase6_scheduler_policy(1, 600),
        phase6_budget(1, 3, 2, 2),
    )
    .expect("runtime should initialize");
    let preflight_error = runtime.run_cycle(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(
                202401,
                vec![
                    "runtime-preflight-a".to_owned(),
                    "runtime-preflight-b".to_owned(),
                ],
            )]),
        ),
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

    let state = runtime.state();
    assert_eq!(state.last_successful_tick_epoch_seconds, None);
    assert_eq!(state.last_observed_now_epoch_seconds, Some(1_700_000_000));
    assert_eq!(state.total_cycles, 1);
    assert_eq!(state.executed_cycles, 0);
    assert_eq!(state.deferred_cycles, 0);
    assert_eq!(state.fail_closed_cycles, 1);
    assert_eq!(
        state.last_reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE
    );
}

pub(super) fn run_spec_c32_phase6_scheduler_runtime_clock_regression_fails_closed_and_preserves_checkpoint(
) {
    let owner_did = "kamn:did:owner:phase6-runtime-clock";
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202601, false))
        .expect("partition should register");

    let mut recent_message = m8_message_input(owner_did, "runtime-clock-message", 1_699_999_990);
    recent_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(recent_message)
        .expect("message should register");

    let mut runtime = DataLayerM10Phase6SchedulerRuntime::new(
        phase6_scheduler_policy(2, 2_000_000_000),
        phase6_budget(2, 2, 1, 1),
    )
    .expect("runtime should initialize");
    runtime
        .run_cycle(
            &mut m8_registry,
            &mut m10_registry,
            phase6_request(
                owner_did,
                BTreeMap::from([(202601, vec!["runtime-clock-message".to_owned()])]),
            ),
        )
        .expect("first deferred cycle should succeed");

    let mut regressed_request = phase6_request(
        owner_did,
        BTreeMap::from([(202601, vec!["runtime-clock-message".to_owned()])]),
    );
    regressed_request.now_epoch_seconds = 1_699_999_999;
    let regressed = runtime.run_cycle(&mut m8_registry, &mut m10_registry, regressed_request);
    assert!(matches!(
        regressed,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6SchedulerSignal {
                field: "now_epoch_seconds",
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE,
            }
        )
    ));

    let state = runtime.state();
    assert_eq!(state.last_successful_tick_epoch_seconds, None);
    assert_eq!(state.last_observed_now_epoch_seconds, Some(1_700_000_000));
    assert_eq!(state.total_cycles, 2);
    assert_eq!(state.executed_cycles, 0);
    assert_eq!(state.deferred_cycles, 1);
    assert_eq!(state.fail_closed_cycles, 1);
    assert_eq!(
        state.last_reason_code,
        DATA_LAYER_M10_PHASE6_SCHEDULER_SIGNAL_INVALID_REASON_CODE
    );
}
