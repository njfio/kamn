use super::{
    auth::map_anti_spam_rejection_to_reasoned_error, drain_service_api_relay_spool_entries,
    project_service_api_relayed_message_statuses, AntiSpamRejection,
    REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
    REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
    REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED, REASON_CODE_INGRESS_SENDER_SUSPENDED,
};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn anti_spam_rate_limit_rejection_maps_to_sender_rate_limit_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::RateLimitExceeded {
        limit: 3,
        observed: 3,
        window_seconds: 5,
    });
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED
    );
    assert!(error.message.contains("observed=3"));
}

#[test]
fn anti_spam_sender_suspension_maps_to_sender_suspended_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::SenderSuspended {
        until_unix: 123_456,
    });
    assert_eq!(error.reason_code, REASON_CODE_INGRESS_SENDER_SUSPENDED);
    assert!(error.message.contains("123456"));
}

#[test]
fn anti_spam_insufficient_deposit_maps_to_sender_deposit_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::InsufficientDeposit {
        required: 9,
        provided: 4,
    });
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT
    );
    assert!(error.message.contains("required=9"));
    assert!(error.message.contains("provided=4"));
}

#[test]
fn anti_spam_duplicate_message_maps_to_sender_duplicate_reason_code() {
    let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::DuplicateMessageId(
        "message-1".to_owned(),
    ));
    assert_eq!(
        error.reason_code,
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID
    );
    assert!(error.message.contains("message-1"));
}

#[test]
fn service_api_relay_projection_marks_created_messages_as_relayed() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-state-{unique_suffix}.json"
    ));
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-projection-created-1":{
      "message_id":"msg-projection-created-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"hello\"}"
    },
    "msg-projection-delivered-1":{
      "message_id":"msg-projection-delivered-1",
      "status":"delivered",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"already\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state fixture should write");
    let message_ids = vec![
        "msg-projection-created-1".to_owned(),
        "msg-projection-delivered-1".to_owned(),
    ];
    let projected_count =
        project_service_api_relayed_message_statuses(state_file.to_str(), message_ids.as_slice())
            .expect("projection should succeed");
    assert_eq!(
        projected_count, 1,
        "exactly one created record should be promoted to relayed"
    );

    let payload = std::fs::read_to_string(state_file.as_path())
        .expect("projected state should stay readable");
    let state_json: Value = serde_json::from_str(payload.as_str()).expect("state should parse");
    assert_eq!(
        state_json["messages"]["msg-projection-created-1"]["status"],
        "relayed"
    );
    assert_eq!(
        state_json["messages"]["msg-projection-delivered-1"]["status"],
        "delivered"
    );
    let _ = std::fs::remove_file(state_file);
}

#[test]
fn service_api_relay_projection_is_idempotent_for_relayed_messages() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-idempotent-{unique_suffix}.json"
    ));
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-projection-relayed-1":{
      "message_id":"msg-projection-relayed-1",
      "status":"relayed",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"stable\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state fixture should write");
    let message_ids = vec!["msg-projection-relayed-1".to_owned()];
    let projected_count =
        project_service_api_relayed_message_statuses(state_file.to_str(), message_ids.as_slice())
            .expect("projection should succeed");
    assert_eq!(
        projected_count, 0,
        "already relayed records must remain unchanged"
    );

    let payload = std::fs::read_to_string(state_file.as_path())
        .expect("projected state should stay readable");
    let state_json: Value = serde_json::from_str(payload.as_str()).expect("state should parse");
    assert_eq!(
        state_json["messages"]["msg-projection-relayed-1"]["status"],
        "relayed"
    );
    let _ = std::fs::remove_file(state_file);
}

#[test]
fn service_api_relay_spool_drain_errors_for_non_not_found_paths() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let directory_path = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-spool-dir-{unique_suffix}"
    ));
    std::fs::create_dir_all(directory_path.as_path()).expect("directory fixture should exist");
    let error = drain_service_api_relay_spool_entries(directory_path.to_str())
        .expect_err("directory path must fail relay spool open");
    assert!(
        error.contains("service api relay spool read failed"),
        "relay spool path errors should fail closed with read failure marker"
    );
    let _ = std::fs::remove_dir(directory_path);
}

#[test]
fn service_api_relay_projection_missing_state_file_is_noop() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let missing_state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-missing-{unique_suffix}.json"
    ));
    let message_ids = vec!["msg-projection-missing-1".to_owned()];
    let projected_count = project_service_api_relayed_message_statuses(
        missing_state_file.to_str(),
        message_ids.as_slice(),
    )
    .expect("missing state file should be treated as no-op");
    assert_eq!(projected_count, 0);
}

#[test]
fn service_api_relay_projection_errors_for_non_not_found_paths() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let directory_path = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-dir-{unique_suffix}"
    ));
    std::fs::create_dir_all(directory_path.as_path()).expect("directory fixture should exist");
    let message_ids = vec!["msg-projection-error-1".to_owned()];
    let error = project_service_api_relayed_message_statuses(
        directory_path.to_str(),
        message_ids.as_slice(),
    )
    .expect_err("directory path should fail state file read");
    assert!(
        error.contains("service api state file read failed"),
        "state projection should fail closed for non-not-found read failures"
    );
    let _ = std::fs::remove_dir(directory_path);
}

#[test]
fn service_api_relay_projection_does_not_rewrite_when_no_records_promoted() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-no-rewrite-{unique_suffix}.json"
    ));
    let initial_payload = r#"{"schema_version":"kamn.runtime.service-api-message-store.v2","messages":{"msg-projection-stable-1":{"message_id":"msg-projection-stable-1","status":"relayed","channel_id":null,"sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"stable\"}"}},"channel_messages":{},"tasks":{},"escrows":{}}"#;
    std::fs::write(state_file.as_path(), initial_payload).expect("state fixture should write");

    let message_ids = vec!["msg-projection-stable-1".to_owned()];
    let projected_count =
        project_service_api_relayed_message_statuses(state_file.to_str(), message_ids.as_slice())
            .expect("projection should succeed");
    assert_eq!(projected_count, 0);
    let final_payload = std::fs::read_to_string(state_file.as_path())
        .expect("state payload should remain readable");
    assert_eq!(
        final_payload, initial_payload,
        "non-promoting projections must not rewrite the state file"
    );
    let _ = std::fs::remove_file(state_file);
}
