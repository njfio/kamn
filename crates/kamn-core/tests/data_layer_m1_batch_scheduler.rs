use kamn_core::{
    evaluate_data_layer_m1_batch_trigger, DataLayerM1BatchSchedulerError,
    DataLayerM1BatchSchedulerPolicy, DataLayerM1BatchTriggerDecision,
    DataLayerM1PendingBatchMessage, DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED,
    DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD,
};

fn pending_message(
    message_id: &str,
    created_at_unix_seconds: u64,
) -> DataLayerM1PendingBatchMessage {
    DataLayerM1PendingBatchMessage {
        message_id: message_id.to_owned(),
        content_hash: "sha256:pending".to_owned(),
        created_at_unix_seconds,
    }
}

#[test]
fn spec_c01_scheduler_defers_when_count_and_window_thresholds_are_not_met() {
    let policy = DataLayerM1BatchSchedulerPolicy::new(3, 60).expect("policy should be valid");
    let now = 1_900_000_000;
    let pending = vec![
        pending_message("msg-c01-a", now - 20),
        pending_message("msg-c01-b", now - 10),
    ];

    let decision = evaluate_data_layer_m1_batch_trigger(&policy, &pending, now)
        .expect("trigger evaluation should succeed");
    assert_eq!(
        decision,
        DataLayerM1BatchTriggerDecision::Deferred {
            reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_DEFERRED,
            pending_count: 2,
            oldest_pending_age_seconds: 20,
        }
    );
}

#[test]
fn spec_c02_scheduler_triggers_deterministically_for_count_and_window_thresholds() {
    let now = 1_900_000_000;

    let count_policy =
        DataLayerM1BatchSchedulerPolicy::new(2, 120).expect("policy should be valid");
    let count_pending = vec![
        pending_message("msg-c02-count-a", now - 5),
        pending_message("msg-c02-count-b", now - 1),
    ];
    let count_decision = evaluate_data_layer_m1_batch_trigger(&count_policy, &count_pending, now)
        .expect("count-threshold evaluation should succeed");
    assert_eq!(
        count_decision,
        DataLayerM1BatchTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_COUNT_THRESHOLD,
            pending_count: 2,
            oldest_pending_age_seconds: 5,
        }
    );

    let window_policy =
        DataLayerM1BatchSchedulerPolicy::new(10, 30).expect("policy should be valid");
    let window_pending = vec![pending_message("msg-c02-window-a", now - 40)];
    let window_decision =
        evaluate_data_layer_m1_batch_trigger(&window_policy, &window_pending, now)
            .expect("window-threshold evaluation should succeed");
    assert_eq!(
        window_decision,
        DataLayerM1BatchTriggerDecision::Triggered {
            reason_code: DATA_LAYER_M1_BATCH_TRIGGER_REASON_CODE_WINDOW_THRESHOLD,
            pending_count: 1,
            oldest_pending_age_seconds: 40,
        }
    );
}

#[test]
fn spec_c04_scheduler_fails_closed_for_invalid_policy_or_future_pending_timestamps() {
    let invalid_policy = DataLayerM1BatchSchedulerPolicy::new(0, 60)
        .expect_err("zero message threshold should fail closed");
    assert_eq!(
        invalid_policy,
        DataLayerM1BatchSchedulerError::InvalidThreshold {
            field: "max_messages_per_batch",
            detail: "must be greater than zero".to_owned(),
        }
    );

    let valid_policy = DataLayerM1BatchSchedulerPolicy::new(2, 60).expect("policy should be valid");
    let now = 1_900_000_000;
    let future_pending = vec![pending_message("msg-c04-future", now + 1)];

    let invalid_pending = evaluate_data_layer_m1_batch_trigger(&valid_policy, &future_pending, now)
        .expect_err("future-created pending message should fail closed");
    assert_eq!(
        invalid_pending,
        DataLayerM1BatchSchedulerError::InvalidPendingMessage {
            field: "created_at_unix_seconds",
            detail: "pending message timestamp is in the future".to_owned(),
        }
    );
}
