use kamn_data_layer::{
    data_layer_m10_evaluate_phase6_execution_tick_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_preflight_budget_policy,
    data_layer_m10_evaluate_phase6_scheduler_trigger_policy,
    data_layer_m10_validate_phase6_execution_budget_policy,
    data_layer_m10_validate_phase6_scheduler_trigger_policy_config,
    DataLayerM10Phase6PolicyBudget, DataLayerM10Phase6PolicyBudgetDecision,
    DataLayerM10Phase6PolicyReportCounts, DataLayerM10Phase6SchedulerSignalPolicy,
    DataLayerM10Phase6SchedulerTriggerPolicy, DataLayerM10Phase6TriggerPolicyDecision,
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
}
