use kamn_data_layer::{
    DataLayerM11HardeningMatrix, DataLayerM11OperatorReadinessDecision,
    DataLayerM11ScenarioDefinition, DataLayerM11ScenarioDomain, DataLayerM11ScenarioOutcomeInput,
    DataLayerM11ScenarioSeverity, DataLayerM11ScenarioStatus,
};

#[test]
fn integration_m11_hardening_readiness_go_when_required_scenarios_pass() {
    let mut matrix = DataLayerM11HardeningMatrix::new();
    matrix
        .register_scenario(DataLayerM11ScenarioDefinition {
            scenario_id: "security.authz-negative-matrix".to_owned(),
            domain: DataLayerM11ScenarioDomain::Security,
            severity: DataLayerM11ScenarioSeverity::Critical,
            required: true,
        })
        .expect("security scenario should register");
    matrix
        .record_outcome(DataLayerM11ScenarioOutcomeInput {
            scenario_id: "security.authz-negative-matrix".to_owned(),
            status: DataLayerM11ScenarioStatus::Passed,
            evidence_marker: "evidence:security".to_owned(),
        })
        .expect("security outcome should record");
    let report = matrix
        .evaluate_operator_readiness()
        .expect("operator readiness should evaluate");
    assert_eq!(report.decision, DataLayerM11OperatorReadinessDecision::Go);
}
