use super::*;

pub(super) fn run_spec_c33_phase6_runtime_evidence_bundle_projects_applied_cycle_with_deterministic_artifacts(
) {
    let evidence_input = DataLayerM10Phase6RuntimeEvidenceInput {
        owner_did: "kamn:did:owner:phase6-evidence-alpha".to_owned(),
        cycle_report: DataLayerM10Phase6SchedulerCycleReport {
            trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
                due_candidate_count: 2,
                elapsed_since_last_tick_seconds: 120,
            },
            execution_report: Some(DataLayerM10Phase6ExecutionTickReport {
                owner_did: "kamn:did:owner:phase6-evidence-alpha".to_owned(),
                due_candidate_count: 2,
                shredded_message_ids: vec!["message-a".to_owned(), "message-b".to_owned()],
                projection_reports: Vec::new(),
                archived_entries: vec![
                    DataLayerM10ArchivalIndexEntry {
                        partition_month_id: 202402,
                        partition_name: "messages_2024_02".to_owned(),
                        archived_object_uri:
                            "s3://kamn-archive/messages/messages_2024_02.parquet.zst".to_owned(),
                        archive_format_marker: DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
                        checksum_marker: "sha256:messages_2024_02:202402".to_owned(),
                        lifecycle_status: DataLayerM10PartitionStatus::Archived,
                    },
                    DataLayerM10ArchivalIndexEntry {
                        partition_month_id: 202401,
                        partition_name: "messages_2024_01".to_owned(),
                        archived_object_uri:
                            "s3://kamn-archive/messages/messages_2024_01.parquet.zst".to_owned(),
                        archive_format_marker: DATA_LAYER_M10_ARCHIVE_FORMAT_PARQUET_ZSTD,
                        checksum_marker: "sha256:messages_2024_01:202401".to_owned(),
                        lifecycle_status: DataLayerM10PartitionStatus::Archived,
                    },
                ],
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
            }),
            budget_report: Some(DataLayerM10Phase6ExecutionTickBudgetReport {
                decision: DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget,
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
                due_candidate_count: 2,
                shredded_message_count: 2,
                projection_report_count: 0,
                archived_entry_count: 2,
            }),
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
        },
        runtime_state: phase6_runtime_state(
            7,
            4,
            2,
            1,
            DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
        ),
    };
    let evidence: DataLayerM10Phase6RuntimeEvidenceBundle =
        data_layer_m10_project_phase6_runtime_evidence_bundle(evidence_input)
            .expect("applied evidence projection should succeed");
    assert_eq!(
        evidence.reason_code,
        DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_APPLIED_REASON_CODE
    );
    assert_eq!(
        evidence.archived_partition_names,
        vec!["messages_2024_01".to_owned(), "messages_2024_02".to_owned()]
    );
    assert_eq!(
        evidence.archived_object_uris,
        vec![
            "s3://kamn-archive/messages/messages_2024_01.parquet.zst".to_owned(),
            "s3://kamn-archive/messages/messages_2024_02.parquet.zst".to_owned(),
        ]
    );
    assert_eq!(
        evidence.budget_reason_code,
        Some("m10_phase6_execution_budget_within_limit")
    );
    assert_eq!(evidence.runtime_total_cycles, 7);
}

pub(super) fn run_spec_c34_phase6_runtime_evidence_bundle_projects_deferred_cycle_with_empty_artifacts(
) {
    let evidence_input = DataLayerM10Phase6RuntimeEvidenceInput {
        owner_did: "kamn:did:owner:phase6-evidence-beta".to_owned(),
        cycle_report: DataLayerM10Phase6SchedulerCycleReport {
            trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
                due_candidate_count: 1,
                elapsed_since_last_tick_seconds: 59,
            },
            execution_report: None,
            budget_report: None,
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        },
        runtime_state: DataLayerM10Phase6SchedulerRuntimeState {
            last_successful_tick_epoch_seconds: None,
            last_observed_now_epoch_seconds: Some(1_700_000_000),
            total_cycles: 3,
            executed_cycles: 1,
            deferred_cycles: 2,
            fail_closed_cycles: 0,
            last_reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        },
    };
    let evidence = data_layer_m10_project_phase6_runtime_evidence_bundle(evidence_input)
        .expect("deferred evidence projection should succeed");
    assert_eq!(
        evidence.reason_code,
        DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_DEFERRED_REASON_CODE
    );
    assert_eq!(evidence.budget_reason_code, None);
    assert_eq!(evidence.due_candidate_count, 1);
    assert!(evidence.archived_partition_names.is_empty());
    assert!(evidence.archived_object_uris.is_empty());
}

pub(super) fn run_spec_c35_phase6_runtime_evidence_bundle_fails_closed_when_applied_payload_is_incomplete(
) {
    let evidence_input = DataLayerM10Phase6RuntimeEvidenceInput {
        owner_did: "kamn:did:owner:phase6-evidence-gamma".to_owned(),
        cycle_report: DataLayerM10Phase6SchedulerCycleReport {
            trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision::Triggered {
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DUE_THRESHOLD_REASON_CODE,
                due_candidate_count: 1,
                elapsed_since_last_tick_seconds: 100,
            },
            execution_report: None,
            budget_report: None,
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
        },
        runtime_state: phase6_runtime_state(
            2,
            1,
            1,
            0,
            DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_APPLIED_REASON_CODE,
        ),
    };
    let invalid = data_layer_m10_project_phase6_runtime_evidence_bundle(evidence_input);
    assert!(matches!(
        invalid,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "cycle_report",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            }
        )
    ));
}

pub(super) fn run_spec_c36_phase6_runtime_evidence_bundle_fails_closed_when_deferred_payload_contains_execution_data(
) {
    let evidence_input = DataLayerM10Phase6RuntimeEvidenceInput {
        owner_did: "kamn:did:owner:phase6-evidence-delta".to_owned(),
        cycle_report: DataLayerM10Phase6SchedulerCycleReport {
            trigger_decision: DataLayerM10Phase6SchedulerTriggerDecision::Deferred {
                reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_TRIGGER_DEFERRED_REASON_CODE,
                due_candidate_count: 0,
                elapsed_since_last_tick_seconds: 10,
            },
            execution_report: Some(DataLayerM10Phase6ExecutionTickReport {
                owner_did: "kamn:did:owner:phase6-evidence-delta".to_owned(),
                due_candidate_count: 0,
                shredded_message_ids: Vec::new(),
                projection_reports: Vec::new(),
                archived_entries: Vec::new(),
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
            }),
            budget_report: Some(DataLayerM10Phase6ExecutionTickBudgetReport {
                decision: DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget,
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE,
                due_candidate_count: 0,
                shredded_message_count: 0,
                projection_report_count: 0,
                archived_entry_count: 0,
            }),
            reason_code: DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        },
        runtime_state: phase6_runtime_state(
            5,
            2,
            2,
            1,
            DATA_LAYER_M10_PHASE6_SCHEDULER_CYCLE_DEFERRED_REASON_CODE,
        ),
    };
    let invalid = data_layer_m10_project_phase6_runtime_evidence_bundle(evidence_input);
    assert!(matches!(
        invalid,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6RuntimeEvidenceInput {
                field: "cycle_report",
                reason_code: DATA_LAYER_M10_PHASE6_RUNTIME_EVIDENCE_INPUT_INVALID_REASON_CODE,
            }
        )
    ));
}
