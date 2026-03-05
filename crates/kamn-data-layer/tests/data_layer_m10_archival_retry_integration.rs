use kamn_data_layer::{
    data_layer_m10_project_archival_retry_decision, DataLayerM10ArchivalFailureClass,
    DataLayerM10ArchivalRecoveryAction, DataLayerM10ArchivalRetryPolicy,
    DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE,
};

#[test]
fn integration_m10_retry_projection_schedules_retry_for_transient_failure() {
    let decision = data_layer_m10_project_archival_retry_decision(
        1_735_700_000,
        1,
        DataLayerM10ArchivalFailureClass::Transient,
        DataLayerM10ArchivalRetryPolicy {
            max_attempts: 3,
            base_backoff_seconds: 30,
            max_backoff_seconds: 300,
        },
    )
    .expect("transient failure under max attempts should schedule retry");

    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::RetryScheduled
    );
    assert_eq!(
        decision.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
    );
}
