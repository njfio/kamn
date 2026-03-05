use super::*;

const OWNER_PHASE6_GAMMA: &str = "kamn:did:owner:phase6-gamma";
const OWNER_PHASE6_DELTA: &str = "kamn:did:owner:phase6-delta";
const OWNER_BUDGET_ALPHA: &str = "kamn:did:owner:phase6-budget-alpha";
const OWNER_BUDGET_BETA: &str = "kamn:did:owner:phase6-budget-beta";

pub(super) fn run_spec_c18_phase6_orchestration_tick_reports_zero_due_without_archival() {
    let owner_did = OWNER_PHASE6_GAMMA;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202601, false))
        .expect("partition should register");

    let mut recent_message = m8_message_input(owner_did, "message-z", 1_699_999_900);
    recent_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    m8_registry
        .register_message(recent_message)
        .expect("message should register");

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202601, vec!["message-z".to_owned()])]),
        ),
    )
    .expect("phase6 execution tick should succeed");

    assert_eq!(report.due_candidate_count, 0);
    assert!(report.shredded_message_ids.is_empty());
    assert_eq!(report.projection_reports.len(), 1);
    assert!(!report.projection_reports[0].all_messages_shredded);
    assert!(report.archived_entries.is_empty());
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE
    );
}

pub(super) fn run_spec_c19_phase6_orchestration_tick_fails_closed_on_legal_hold_and_empty_projection_entries(
) {
    let owner_did = OWNER_PHASE6_DELTA;

    let mut hold_m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut hold_m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    hold_m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    let mut held_message = m8_message_input(owner_did, "message-held", 1_699_700_000);
    held_message.retention_class = DataLayerM8RetentionClass::Ephemeral;
    hold_m8_registry
        .register_message(held_message)
        .expect("message should register");
    hold_m8_registry
        .set_legal_hold(DataLayerM8LegalHoldRequest {
            requester_owner_did: owner_did.to_owned(),
            owner_did: owner_did.to_owned(),
            message_id: "message-held".to_owned(),
            legal_hold_active: true,
        })
        .expect("legal hold should apply");
    let legal_hold_error = data_layer_m10_execute_phase6_orchestration_tick(
        &mut hold_m8_registry,
        &mut hold_m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202401, vec!["message-held".to_owned()])]),
        ),
    );
    assert!(matches!(
        legal_hold_error,
        Err(DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_LEGAL_HOLD_ACTIVE_REASON_CODE,
            ..
        })
    ));

    let mut empty_m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut empty_m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    empty_m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    empty_m8_registry
        .register_message(m8_message_input(owner_did, "message-empty", 1_699_999_000))
        .expect("message should register");
    let empty_projection_error = data_layer_m10_execute_phase6_orchestration_tick(
        &mut empty_m8_registry,
        &mut empty_m10_registry,
        phase6_request(owner_did, BTreeMap::from([(202401, Vec::new())])),
    );
    assert!(matches!(
        empty_projection_error,
        Err(DataLayerM10PartitionLifecycleError::Phase6ExecutionFailed {
            reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_PROJECTION_INPUT_INVALID_REASON_CODE,
            ..
        })
    ));
}

pub(super) fn run_spec_c20_phase6_execution_tick_budget_within_limits_and_exceeded_paths_are_deterministic(
) {
    let owner_did = OWNER_BUDGET_ALPHA;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("budget-a", 1_699_800_000_u64),
        ("budget-b", 1_699_800_100_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([(202401, vec!["budget-a".to_owned(), "budget-b".to_owned()])]),
        ),
    )
    .expect("phase6 execution should succeed");

    let within_budget =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(2, 2, 1, 1))
            .expect("within-budget evaluation should succeed");
    assert_eq!(
        within_budget.decision,
        DataLayerM10Phase6ExecutionBudgetDecision::WithinBudget
    );
    assert_eq!(
        within_budget.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_WITHIN_LIMIT_REASON_CODE
    );

    let due_exceeded =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(1, 3, 2, 2))
            .expect("due-exceeded evaluation should succeed");
    assert_eq!(
        due_exceeded.decision,
        DataLayerM10Phase6ExecutionBudgetDecision::Exceeded
    );
    assert_eq!(
        due_exceeded.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_DUE_CANDIDATES_EXCEEDED_REASON_CODE
    );

    let shredded_exceeded =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(3, 1, 2, 2))
            .expect("shredded-exceeded evaluation should succeed");
    assert_eq!(
        shredded_exceeded.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_SHREDDED_MESSAGES_EXCEEDED_REASON_CODE
    );
}

pub(super) fn run_spec_c21_phase6_execution_tick_budget_projection_and_archive_limits_fail_closed()
{
    let owner_did = OWNER_BUDGET_BETA;
    let mut m8_registry = DataLayerM8ComplianceRegistry::new();
    let mut m10_registry = DataLayerM10PartitionLifecycleRegistry::new();
    m10_registry
        .register_partition(partition_input(202401, false))
        .expect("partition should register");
    m10_registry
        .register_partition(partition_input(202402, false))
        .expect("partition should register");

    for (message_id, created_at) in [
        ("proj-a", 1_699_700_000_u64),
        ("proj-b", 1_699_700_100_u64),
        ("proj-c", 1_699_700_200_u64),
    ] {
        let mut input = m8_message_input(owner_did, message_id, created_at);
        input.retention_class = DataLayerM8RetentionClass::Ephemeral;
        m8_registry
            .register_message(input)
            .expect("message should register");
    }

    let report = data_layer_m10_execute_phase6_orchestration_tick(
        &mut m8_registry,
        &mut m10_registry,
        phase6_request(
            owner_did,
            BTreeMap::from([
                (202401, vec!["proj-a".to_owned(), "proj-b".to_owned()]),
                (202402, vec!["proj-c".to_owned()]),
            ]),
        ),
    )
    .expect("phase6 execution should succeed");

    let projection_exceeded =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(5, 5, 1, 5))
            .expect("projection-exceeded evaluation should succeed");
    assert_eq!(
        projection_exceeded.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_PROJECTIONS_EXCEEDED_REASON_CODE
    );

    let archive_exceeded =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(5, 5, 5, 1))
            .expect("archive-exceeded evaluation should succeed");
    assert_eq!(
        archive_exceeded.reason_code,
        DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_ARCHIVE_ENTRIES_EXCEEDED_REASON_CODE
    );
}

pub(super) fn run_spec_c22_phase6_execution_tick_budget_invalid_limits_fail_closed() {
    let report = kamn_core::DataLayerM10Phase6ExecutionTickReport {
        owner_did: "kamn:did:owner:phase6-budget-invalid".to_owned(),
        due_candidate_count: 0,
        shredded_message_ids: Vec::new(),
        projection_reports: Vec::new(),
        archived_entries: Vec::new(),
        reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_APPLIED_REASON_CODE,
    };

    let invalid_budget =
        data_layer_m10_evaluate_phase6_execution_tick_budget(&report, phase6_budget(0, 1, 1, 1));
    assert!(matches!(
        invalid_budget,
        Err(
            DataLayerM10PartitionLifecycleError::InvalidPhase6ExecutionBudget {
                field: "max_due_candidates",
                reason_code: DATA_LAYER_M10_PHASE6_EXECUTION_BUDGET_INVALID_REASON_CODE,
            }
        )
    ));
}
