use super::*;

pub(super) fn run_spec_c12_transient_archival_failure_projects_deterministic_retry_window() {
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 4,
        base_backoff_seconds: 5,
        max_backoff_seconds: 60,
    };
    let decision = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
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
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 16,
        base_backoff_seconds: 4,
        max_backoff_seconds: 20,
    };
    let decision = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
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
    let policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 3,
        base_backoff_seconds: 5,
        max_backoff_seconds: 30,
    };
    let exhausted = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        3,
        DataLayerM10ArchivalFailureClass::Transient,
        policy,
    )
    .expect("exhausted transient should project fail-closed");
    assert_eq!(
        exhausted.action,
        DataLayerM10ArchivalRecoveryAction::FailClosed
    );
    assert_eq!(exhausted.next_attempt, None);
    assert_eq!(exhausted.retry_backoff_seconds, None);
    assert_eq!(exhausted.retry_after_unix_seconds, None);
    assert_eq!(exhausted.attempts_remaining, 0);
    assert_eq!(
        exhausted.reason_code,
        DATA_LAYER_M10_ARCHIVAL_RETRY_EXHAUSTED_REASON_CODE
    );

    let permanent = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
        1,
        DataLayerM10ArchivalFailureClass::Permanent,
        policy,
    )
    .expect("permanent failure should project fail-closed");
    assert_eq!(
        permanent.action,
        DataLayerM10ArchivalRecoveryAction::FailClosed
    );
    assert_eq!(permanent.next_attempt, None);
    assert_eq!(permanent.retry_backoff_seconds, None);
    assert_eq!(permanent.retry_after_unix_seconds, None);
    assert_eq!(permanent.attempts_remaining, 0);
    assert_eq!(
        permanent.reason_code,
        DATA_LAYER_M10_ARCHIVAL_FAILURE_PERMANENT_REASON_CODE
    );
}

pub(super) fn run_spec_c15_archival_retry_policy_and_attempt_validation_fail_closed() {
    let invalid_policy = DataLayerM10ArchivalRetryPolicy {
        max_attempts: 0,
        base_backoff_seconds: 5,
        max_backoff_seconds: 30,
    };
    let invalid_policy_error = data_layer_m10_project_archival_retry_decision(
        1_700_000_000,
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
        1_700_000_000,
        0,
        DataLayerM10ArchivalFailureClass::Transient,
        DataLayerM10ArchivalRetryPolicy {
            max_attempts: 3,
            base_backoff_seconds: 5,
            max_backoff_seconds: 30,
        },
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
