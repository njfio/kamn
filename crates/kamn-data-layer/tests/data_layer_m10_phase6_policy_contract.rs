use kamn_data_layer::{
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_trigger_policy,
    data_layer_m10_project_phase6_scheduler_budget_overflow_policy_error,
    data_layer_m10_project_phase6_scheduler_cycle_policy_report,
    data_layer_m10_validate_phase6_execution_budget_policy,
    data_layer_m10_validate_phase6_scheduler_runtime_clock_signal,
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config, DataLayerM10Phase6PolicyBudget,
    DataLayerM10Phase6PolicyBudgetDecision, DataLayerM10Phase6PolicyReportCounts,
    DataLayerM10Phase6SchedulerBudgetOverflowPolicyProjection,
    DataLayerM10Phase6SchedulerBudgetOverflowStage, DataLayerM10Phase6SchedulerCyclePolicyReport,
    DataLayerM10Phase6SchedulerSignalPolicy, DataLayerM10Phase6SchedulerTriggerPolicy,
    DataLayerM10Phase6TriggerPolicyDecision,
};

#[test]
fn contract_phase6_policy_budget_and_trigger_surfaces_are_exported() {
    let budget = DataLayerM10Phase6PolicyBudget {
        max_due_candidates: 2,
        max_shredded_messages: 2,
        max_projection_reports: 2,
        max_archived_entries: 2,
    };
    let counts = DataLayerM10Phase6PolicyReportCounts {
        due_candidate_count: 3,
        shredded_message_count: 1,
        projection_report_count: 1,
        archived_entry_count: 1,
    };
    let budget_report = data_layer_m10_evaluate_phase6_execution_tick_budget_policy(counts, budget)
        .expect("policy evaluation should succeed");
    assert_eq!(
        budget_report.decision,
        DataLayerM10Phase6PolicyBudgetDecision::Exceeded
    );
    let preflight_budget_report =
        data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy(1, 3, budget)
            .expect("scheduler preflight policy evaluation should succeed");
    assert_eq!(
        preflight_budget_report.decision,
        DataLayerM10Phase6PolicyBudgetDecision::Exceeded
    );

    let trigger = data_layer_m10_evaluate_phase6_scheduler_trigger_policy(
        DataLayerM10Phase6SchedulerTriggerPolicy {
            due_candidate_trigger_threshold: 2,
            max_tick_interval_seconds: 60,
        },
        DataLayerM10Phase6SchedulerSignalPolicy {
            due_candidate_count: 2,
            last_tick_epoch_seconds: Some(1_700_000_000),
            now_epoch_seconds: 1_700_000_001,
        },
    )
    .expect("trigger evaluation should succeed");
    assert!(matches!(
        trigger,
        DataLayerM10Phase6TriggerPolicyDecision::Triggered { .. }
    ));
    data_layer_m10_validate_phase6_execution_budget_policy(budget)
        .expect("budget validator should accept positive limits");
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config(
        DataLayerM10Phase6SchedulerTriggerPolicy {
            due_candidate_trigger_threshold: 1,
            max_tick_interval_seconds: 60,
        },
    )
    .expect("scheduler policy validator should accept positive thresholds");
    data_layer_m10_validate_phase6_scheduler_runtime_clock_signal(
        1_700_000_010,
        Some(1_700_000_000),
    )
    .expect("runtime clock validator should accept non-regressed now clock");

    let deferred_cycle_report: DataLayerM10Phase6SchedulerCyclePolicyReport<(), ()> =
        data_layer_m10_project_phase6_scheduler_cycle_policy_report(
            DataLayerM10Phase6TriggerPolicyDecision::Deferred {
                reason_code: "m10_phase6_scheduler_trigger_deferred",
                due_candidate_count: 1,
                elapsed_since_last_tick_seconds: 10,
            },
            None,
            None,
        );
    assert_eq!(
        deferred_cycle_report.reason_code,
        "m10_phase6_scheduler_cycle_deferred"
    );

    let preflight_overflow: Option<DataLayerM10Phase6SchedulerBudgetOverflowPolicyProjection> =
        data_layer_m10_project_phase6_scheduler_budget_overflow_policy_error(
            preflight_budget_report,
            DataLayerM10Phase6SchedulerBudgetOverflowStage::Preflight,
        );
    let preflight_overflow = preflight_overflow.expect("preflight overflow should project");
    assert_eq!(
        preflight_overflow.reason_code,
        "m10_phase6_execution_budget_projections_exceeded"
    );
    assert_eq!(
        preflight_overflow.detail,
        "due=1,shredded=1,projections=3,archives=3"
    );
}
