use super::*;

const RETRY_NOW_EPOCH_SECONDS: u64 = 1_700_000_000;

fn retry_policy(
    max_attempts: u8,
    base_backoff_seconds: u64,
    max_backoff_seconds: u64,
) -> DataLayerM10ArchivalRetryPolicy {
    DataLayerM10ArchivalRetryPolicy {
        max_attempts,
        base_backoff_seconds,
        max_backoff_seconds,
    }
}

fn assert_fail_closed(
    decision: &DataLayerM10ArchivalRetryDecision,
    expected_reason_code: &'static str,
) {
    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::FailClosed
    );
    assert_eq!(decision.next_attempt, None);
    assert_eq!(decision.retry_backoff_seconds, None);
    assert_eq!(decision.retry_after_unix_seconds, None);
    assert_eq!(decision.attempts_remaining, 0);
    assert_eq!(decision.reason_code, expected_reason_code);
}

pub(super) fn run_spec_c12_transient_archival_failure_projects_deterministic_retry_window() {
    let policy = retry_policy(4, 5, 60);
    let decision = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        2,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("transient failure should project retry");

    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::RetryScheduled
    );
    assert_eq!(decision.current_attempt, 2);
    assert_eq!(decision.next_attempt, Some(3));
    assert_eq!(decision.retry_backoff_seconds, Some(10));
    assert_eq!(decision.retry_after_unix_seconds, Some(1_700_000_010));
    assert_eq!(decision.attempts_remaining, 2);
    assert_eq!(
        decision.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
    );
}

pub(super) fn run_spec_c13_transient_archival_retry_backoff_caps_at_policy_maximum() {
    let policy = retry_policy(16, 4, 20);
    let decision = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        8,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("transient failure should project retry");

    assert_eq!(
        decision.action,
        DataLayerM10ArchivalRecoveryAction::RetryScheduled
    );
    assert_eq!(decision.retry_backoff_seconds, Some(20));
    assert_eq!(decision.retry_after_unix_seconds, Some(1_700_000_020));
    assert_eq!(decision.next_attempt, Some(9));
    assert_eq!(
        decision.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_SCHEDULED_REASON_CODE
    );
}

pub(super) fn run_spec_c14_archival_retry_budget_exhaustion_and_permanent_failure_fail_closed() {
    let policy = retry_policy(3, 5, 30);
    let exhausted = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        3,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("exhausted transient should project fail-closed");
    assert_fail_closed(
        &exhausted,
        DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE,
    );

    let permanent = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        1,
        DataLayerM10ArchivalFailureClass::Permanent,
        policy,
    )
    .expect("permanent failure should project fail-closed");
    assert_fail_closed(
        &permanent,
        DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE,
    );
}

pub(super) fn run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed() {
    let invalid_policy = retry_policy(0, 5, 30);
    let invalid_policy_error = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        1,
        DataLayerM10ArchivalFailureClass::Transient,
        invalid_policy,
    );
    assert!(matches!(
        invalid_policy_error,
        Err(DataLayerM10PartitionLifecycleError::InvalidRetryPolicy {
            field: "max_attempts",
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_POLICY_INVALID_REASON_CODE,
        })
    ));

    let invalid_attempt = data_layer_m10_project_archival_retry_decision(
        RETRY_NOW_EPOCH_SECONDS,
        0,
        DataLayerM10ArchivalFailureClass::Transient,
        retry_policy(3, 5, 30),
    );
    assert!(matches!(
        invalid_attempt,
        Err(DataLayerM10PartitionLifecycleError::InvalidRetryAttempt {
            field: "current_attempt",
            value: 0,
            reason_code: DATA_LAYER_M10_ARCHIVAL_RETRY_ATTEMPT_INVALID_REASON_CODE,
        })
    ));
}
