use kamn_core::{
    DataLayerM11HardeningMatrix, DataLayerM11HardeningMatrixError,
    DataLayerM11OperatorReadinessDecision, DataLayerM11ScenarioDefinition,
    DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput, DataLayerM11ScenarioSeverity,
    DataLayerM11ScenarioStatus, DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE,
    DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE,
    DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE, DATA_LAYER_M11_READINESS_GO_REASON_CODE,
};

fn scenario(
    scenario_id: &str,
    domain: DataLayerM11ScenarioDomain,
    severity: DataLayerM11ScenarioSeverity,
    required: bool,
) -> DataLayerM11ScenarioDefinition {
    DataLayerM11ScenarioDefinition {
        scenario_id: scenario_id.to_owned(),
        domain,
        severity,
        required,
    }
}

fn outcome(
    scenario_id: &str,
    status: DataLayerM11ScenarioStatus,
) -> DataLayerM11ScenarioOutcomeInput {
    DataLayerM11ScenarioOutcomeInput {
        scenario_id: scenario_id.to_owned(),
        status,
        evidence_marker: format!("evidence:{scenario_id}:{status:?}"),
    }
}

#[test]
fn spec_c01_registration_catalog_is_deterministic_and_sorted() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "performance.p95-latency",
            DataLayerM11ScenarioDomain::Performance,
            DataLayerM11ScenarioSeverity::High,
            true,
        ))
        .expect("performance scenario should register");
    matrix
        .register_scenario(scenario(
            "chaos.partition-heal",
            DataLayerM11ScenarioDomain::Chaos,
            DataLayerM11ScenarioSeverity::High,
            true,
        ))
        .expect("chaos scenario should register");
    matrix
        .register_scenario(scenario(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioDomain::Security,
            DataLayerM11ScenarioSeverity::Critical,
            true,
        ))
        .expect("security scenario should register");

    let catalog = matrix.list_scenarios();
    let scenario_ids: Vec<&str> = catalog
        .iter()
        .map(|entry| entry.scenario_id.as_str())
        .collect();
    assert_eq!(
        scenario_ids,
        vec![
            "chaos.partition-heal",
            "performance.p95-latency",
            "security.authz-negative-matrix",
        ]
    );
}

#[test]
fn spec_c02_all_required_pass_results_in_operator_readiness_go() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioDomain::Security,
            DataLayerM11ScenarioSeverity::Critical,
            true,
        ))
        .expect("security scenario should register");
    matrix
        .register_scenario(scenario(
            "chaos.partition-heal",
            DataLayerM11ScenarioDomain::Chaos,
            DataLayerM11ScenarioSeverity::High,
            true,
        ))
        .expect("chaos scenario should register");
    matrix
        .register_scenario(scenario(
            "operations.runbook-signoff",
            DataLayerM11ScenarioDomain::Operations,
            DataLayerM11ScenarioSeverity::Medium,
            true,
        ))
        .expect("operations scenario should register");

    matrix
        .record_outcome(outcome(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioStatus::Passed,
        ))
        .expect("security outcome should record");
    matrix
        .record_outcome(outcome(
            "chaos.partition-heal",
            DataLayerM11ScenarioStatus::Passed,
        ))
        .expect("chaos outcome should record");
    matrix
        .record_outcome(outcome(
            "operations.runbook-signoff",
            DataLayerM11ScenarioStatus::Passed,
        ))
        .expect("operations outcome should record");

    let readiness = matrix
        .evaluate_operator_readiness()
        .expect("readiness evaluation should succeed");
    assert_eq!(
        readiness.decision,
        DataLayerM11OperatorReadinessDecision::Go
    );
    assert_eq!(
        readiness.reason_codes,
        vec![DATA_LAYER_M11_READINESS_GO_REASON_CODE]
    );
}

#[test]
fn spec_c03_critical_failure_for_required_scenario_blocks_readiness() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioDomain::Security,
            DataLayerM11ScenarioSeverity::Critical,
            true,
        ))
        .expect("security scenario should register");
    matrix
        .record_outcome(outcome(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioStatus::Failed,
        ))
        .expect("security outcome should record");

    let readiness = matrix
        .evaluate_operator_readiness()
        .expect("readiness evaluation should succeed");
    assert_eq!(
        readiness.decision,
        DataLayerM11OperatorReadinessDecision::NoGo
    );
    assert_eq!(
        readiness.reason_codes,
        vec![DATA_LAYER_M11_BLOCK_CRITICAL_FAILURE_REASON_CODE]
    );
}

#[test]
fn spec_c04_duplicate_scenario_registration_fails_closed() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "security.authz-negative-matrix",
            DataLayerM11ScenarioDomain::Security,
            DataLayerM11ScenarioSeverity::Critical,
            true,
        ))
        .expect("scenario should register");

    let duplicate = matrix.register_scenario(scenario(
        "security.authz-negative-matrix",
        DataLayerM11ScenarioDomain::Security,
        DataLayerM11ScenarioSeverity::Critical,
        true,
    ));
    assert!(matches!(
        duplicate,
        Err(DataLayerM11HardeningMatrixError::DuplicateScenarioId(id))
        if id == "security.authz-negative-matrix"
    ));
}

#[test]
fn spec_c05_missing_required_scenario_outcomes_are_blocking() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "chaos.partition-heal",
            DataLayerM11ScenarioDomain::Chaos,
            DataLayerM11ScenarioSeverity::High,
            true,
        ))
        .expect("scenario should register");

    let readiness = matrix
        .evaluate_operator_readiness()
        .expect("readiness evaluation should succeed");
    assert_eq!(
        readiness.decision,
        DataLayerM11OperatorReadinessDecision::NoGo
    );
    assert_eq!(
        readiness.reason_codes,
        vec![DATA_LAYER_M11_BLOCK_REQUIRED_INCOMPLETE_REASON_CODE]
    );
    assert_eq!(
        readiness.missing_required_scenario_ids,
        vec!["chaos.partition-heal"]
    );
}

#[test]
fn spec_c06_invalid_status_transition_fails_closed() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(scenario(
            "performance.p95-latency",
            DataLayerM11ScenarioDomain::Performance,
            DataLayerM11ScenarioSeverity::High,
            true,
        ))
        .expect("scenario should register");
    matrix
        .record_outcome(outcome(
            "performance.p95-latency",
            DataLayerM11ScenarioStatus::Passed,
        ))
        .expect("first outcome should record");

    let invalid_transition = matrix.record_outcome(outcome(
        "performance.p95-latency",
        DataLayerM11ScenarioStatus::Failed,
    ));
    assert!(matches!(
        invalid_transition,
        Err(DataLayerM11HardeningMatrixError::InvalidStatusTransition {
            reason_code: DATA_LAYER_M11_INVALID_TRANSITION_REASON_CODE,
            ..
        })
    ));
}
