use kamn_data_layer::{
    data_layer_m10_project_phase6_runtime_evidence_bundle,
    DataLayerM10Phase6PolicyArchivedEntry, DataLayerM10Phase6PolicyBudgetDecision,
    DataLayerM10Phase6PolicyCycleReport, DataLayerM10Phase6PolicyExecutionReport,
    DataLayerM10Phase6PolicyRuntimeEvidenceBundle,
    DataLayerM10Phase6PolicyRuntimeEvidenceError, DataLayerM10Phase6PolicyRuntimeEvidenceInput,
    DataLayerM10Phase6PolicyRuntimeState, DataLayerM10Phase6PolicySchedulerCycleReason,
    DataLayerM10Phase6PolicySchedulerTriggerDecision,
};

#[test]
fn contract_phase6_runtime_evidence_applied_cycle_is_projected_with_sorted_archive_artifacts() {
    let bundle: DataLayerM10Phase6PolicyRuntimeEvidenceBundle =
        data_layer_m10_project_phase6_runtime_evidence_bundle(DataLayerM10Phase6PolicyRuntimeEvidenceInput {
            owner_did: "kamn:did:owner:phase6-evidence-alpha".to_owned(),
            cycle_report: DataLayerM10Phase6PolicyCycleReport {
                trigger_decision: DataLayerM10Phase6PolicySchedulerTriggerDecision::Triggered {
                    reason_code: "m10_phase6_scheduler_trigger_due_threshold",
                    due_candidate_count: 2,
                    elapsed_since_last_tick_seconds: 120,
                },
                execution_report: Some(DataLayerM10Phase6PolicyExecutionReport {
                    owner_did: "kamn:did:owner:phase6-evidence-alpha".to_owned(),
                    due_candidate_count: 2,
                    shredded_message_count: 2,
                    projection_report_count: 0,
                    archived_entries: vec![
                        DataLayerM10Phase6PolicyArchivedEntry {
                            partition_month_id: 202402,
                            partition_name: "messages_2024_02".to_owned(),
                            archived_object_uri:
                                "s3://kamn-archive/messages/messages_2024_02.parquet.zst".to_owned(),
                        },
                        DataLayerM10Phase6PolicyArchivedEntry {
                            partition_month_id: 202401,
                            partition_name: "messages_2024_01".to_owned(),
                            archived_object_uri:
                                "s3://kamn-archive/messages/messages_2024_01.parquet.zst".to_owned(),
                        },
                    ],
                }),
                budget_decision: Some(DataLayerM10Phase6PolicyBudgetDecision::WithinBudget {
                    reason_code: "m10_phase6_execution_budget_within_limit",
                }),
                reason_code: DataLayerM10Phase6PolicySchedulerCycleReason::Applied,
            },
            runtime_state: DataLayerM10Phase6PolicyRuntimeState {
                last_successful_tick_epoch_seconds: Some(1_700_000_000),
                last_observed_now_epoch_seconds: Some(1_700_000_120),
                total_cycles: 7,
                executed_cycles: 4,
                deferred_cycles: 2,
                fail_closed_cycles: 1,
                last_reason_code: "m10_phase6_scheduler_cycle_applied",
            },
        })
        .expect("applied evidence projection should succeed");

    assert_eq!(bundle.reason_code, "m10_phase6_runtime_evidence_applied");
    assert_eq!(
        bundle.archived_partition_names,
        vec!["messages_2024_01".to_owned(), "messages_2024_02".to_owned()]
    );
    assert_eq!(
        bundle.archived_object_uris,
        vec![
            "s3://kamn-archive/messages/messages_2024_01.parquet.zst".to_owned(),
            "s3://kamn-archive/messages/messages_2024_02.parquet.zst".to_owned(),
        ]
    );
}

#[test]
fn contract_phase6_runtime_evidence_deferred_cycle_omits_execution_artifacts() {
    let bundle = data_layer_m10_project_phase6_runtime_evidence_bundle(
        DataLayerM10Phase6PolicyRuntimeEvidenceInput {
            owner_did: "kamn:did:owner:phase6-evidence-beta".to_owned(),
            cycle_report: DataLayerM10Phase6PolicyCycleReport {
                trigger_decision: DataLayerM10Phase6PolicySchedulerTriggerDecision::Deferred {
                    reason_code: "m10_phase6_scheduler_trigger_deferred",
                    due_candidate_count: 1,
                    elapsed_since_last_tick_seconds: 59,
                },
                execution_report: None,
                budget_decision: None,
                reason_code: DataLayerM10Phase6PolicySchedulerCycleReason::Deferred,
            },
            runtime_state: DataLayerM10Phase6PolicyRuntimeState {
                last_successful_tick_epoch_seconds: None,
                last_observed_now_epoch_seconds: Some(1_700_000_000),
                total_cycles: 3,
                executed_cycles: 1,
                deferred_cycles: 2,
                fail_closed_cycles: 0,
                last_reason_code: "m10_phase6_scheduler_cycle_deferred",
            },
        },
    )
    .expect("deferred evidence projection should succeed");

    assert_eq!(bundle.reason_code, "m10_phase6_runtime_evidence_deferred");
    assert_eq!(bundle.due_candidate_count, 1);
    assert_eq!(bundle.budget_reason_code, None);
    assert!(bundle.archived_partition_names.is_empty());
    assert!(bundle.archived_object_uris.is_empty());
}

#[test]
fn contract_phase6_runtime_evidence_fails_closed_for_incomplete_applied_payload() {
    let invalid = data_layer_m10_project_phase6_runtime_evidence_bundle(
        DataLayerM10Phase6PolicyRuntimeEvidenceInput {
            owner_did: "kamn:did:owner:phase6-evidence-gamma".to_owned(),
            cycle_report: DataLayerM10Phase6PolicyCycleReport {
                trigger_decision: DataLayerM10Phase6PolicySchedulerTriggerDecision::Triggered {
                    reason_code: "m10_phase6_scheduler_trigger_due_threshold",
                    due_candidate_count: 1,
                    elapsed_since_last_tick_seconds: 100,
                },
                execution_report: None,
                budget_decision: None,
                reason_code: DataLayerM10Phase6PolicySchedulerCycleReason::Applied,
            },
            runtime_state: DataLayerM10Phase6PolicyRuntimeState {
                last_successful_tick_epoch_seconds: Some(1_700_000_000),
                last_observed_now_epoch_seconds: Some(1_700_000_100),
                total_cycles: 2,
                executed_cycles: 1,
                deferred_cycles: 1,
                fail_closed_cycles: 0,
                last_reason_code: "m10_phase6_scheduler_cycle_applied",
            },
        },
    );

    assert_eq!(
        invalid,
        Err(DataLayerM10Phase6PolicyRuntimeEvidenceError::InvalidInput {
            field: "cycle_report",
            reason_code: "m10_phase6_runtime_evidence_input_invalid",
        })
    );
}
