use super::{
    auth::map_anti_spam_rejection_to_reasoned_error,
    build_service_api_runtime, deterministic_body_tag, drain_service_api_relay_spool_entries,
    message_store::ServiceApiMessageStore,
    project_service_api_relayed_message_statuses, service_api_runtime_worker_threads_for_test,
    state_io::{SERVICE_API_STATE_SQLITE_NAMESPACE, SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY},
    AntiSpamRejection, REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
    REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
    REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED, REASON_CODE_INGRESS_SENDER_SUSPENDED,
};
use kamn_core::SqliteStoreBackend;
use serde_json::Value;
use std::path::Path;
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
fn unit_service_api_runtime_contract_uses_multi_thread_builder() {
    assert!(
        service_api_runtime_worker_threads_for_test() >= 2,
        "service api runtime must provision at least two worker threads"
    );
}

#[test]
fn unit_service_api_runtime_builder_initializes_runtime() {
    let runtime = build_service_api_runtime()
        .expect("service api runtime builder should initialize tokio runtime");
    runtime.block_on(async {});
}

#[test]
fn unit_service_api_deterministic_body_tag_is_stable_and_input_sensitive() {
    let stable_a = deterministic_body_tag(br#"{"message":"hello"}"#);
    let stable_b = deterministic_body_tag(br#"{"message":"hello"}"#);
    let different = deterministic_body_tag(br#"{"message":"goodbye"}"#);
    assert_eq!(stable_a, stable_b);
    assert_ne!(stable_a, different);
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
fn unit_message_store_relay_progress_counts_project_live_message_states() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-progress-counts-{unique_suffix}.json"
    ));
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-progress-created-1":{"message_id":"msg-progress-created-1","status":"created","channel_id":null,"sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"created\"}"},
    "msg-progress-relayed-1":{"message_id":"msg-progress-relayed-1","status":"relayed","channel_id":null,"sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"relayed\"}"},
    "msg-progress-delivered-1":{"message_id":"msg-progress-delivered-1","status":"delivered","channel_id":null,"sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"delivered\"}"},
    "msg-progress-ignored-1":{"message_id":"msg-progress-ignored-1","status":"ignored","channel_id":null,"sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"ignored\"}"}
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("relay progress state fixture should write");
    let mut store = ServiceApiMessageStore::from_optional_state_file(Some(
        state_file.to_string_lossy().to_string(),
    ))
    .expect("state-backed message store should initialize");
    let counts = store
        .relay_progress_counts()
        .expect("relay progress counts should read from state");
    assert_eq!(counts.created_message_count, 1);
    assert_eq!(counts.relayed_message_count, 1);
    assert_eq!(counts.delivered_message_count, 1);
    let _ = std::fs::remove_file(state_file);
}

#[test]
fn integration_message_store_persists_and_recovers_messages_with_sqlite_state_backend() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let sqlite_state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-message-store-{unique_suffix}.sqlite"
    ));
    let sqlite_state_path = sqlite_state_file.to_string_lossy().to_string();

    let created_message_id = {
        let mut store =
            ServiceApiMessageStore::from_optional_state_file(Some(sqlite_state_path.clone()))
                .expect("sqlite-backed message store should initialize");
        let created = store
            .create_message(
                r#"{"recipient_did":"kamn:did:agent:sqlite-recipient","message":"sqlite-store"}"#,
                "api",
                None,
                Some("kamn:did:agent:sqlite-sender"),
                Some("kamn:did:agent:sqlite-recipient"),
            )
            .expect("sqlite-backed message creation should persist");
        created.message_id
    };

    let sqlite_backend = SqliteStoreBackend::open(Path::new(sqlite_state_path.as_str()))
        .expect("sqlite state file should be a valid sqlite database");
    let sqlite_keys = sqlite_backend
        .list_keys(SERVICE_API_STATE_SQLITE_NAMESPACE)
        .expect("sqlite state namespace should be queryable");
    assert_eq!(
        sqlite_keys,
        vec![SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY.to_owned()]
    );
    let sqlite_payload = sqlite_backend
        .get(
            SERVICE_API_STATE_SQLITE_NAMESPACE,
            SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY,
        )
        .expect("sqlite snapshot read should succeed")
        .expect("sqlite snapshot row should exist");
    let sqlite_json: Value =
        serde_json::from_slice(sqlite_payload.as_slice()).expect("sqlite snapshot should be json");
    assert!(
        sqlite_json["messages"]
            .get(created_message_id.as_str())
            .is_some(),
        "sqlite snapshot should include created message id"
    );

    let mut reloaded = ServiceApiMessageStore::from_optional_state_file(Some(sqlite_state_path))
        .expect("sqlite-backed message store should reload");
    let reloaded_payload = reloaded
        .get_message_for_requester(created_message_id.as_str(), None)
        .expect("sqlite-backed message query should succeed")
        .expect("sqlite-backed message should exist");
    assert_eq!(reloaded_payload.message_id, created_message_id);
    assert_eq!(reloaded_payload.status, "created");
    assert_eq!(
        reloaded_payload.sender_did.as_deref(),
        Some("kamn:did:agent:sqlite-sender")
    );
    assert_eq!(
        reloaded_payload.recipient_did.as_deref(),
        Some("kamn:did:agent:sqlite-recipient")
    );

    let _ = std::fs::remove_file(sqlite_state_file);
}

#[test]
fn service_api_relay_projection_marks_created_messages_as_relayed_for_sqlite_state_backend() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let sqlite_state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-projection-{unique_suffix}.sqlite"
    ));
    let sqlite_state_path = sqlite_state_file.to_string_lossy().to_string();

    let created_message_id = {
        let mut store =
            ServiceApiMessageStore::from_optional_state_file(Some(sqlite_state_path.clone()))
                .expect("sqlite-backed message store should initialize");
        let created = store
            .create_message(
                r#"{"recipient_did":"kamn:did:agent:sqlite-projection-recipient","message":"sqlite-projection"}"#,
                "api",
                None,
                Some("kamn:did:agent:sqlite-projection-sender"),
                Some("kamn:did:agent:sqlite-projection-recipient"),
            )
            .expect("sqlite-backed message creation should persist");
        created.message_id
    };

    let projected_count = project_service_api_relayed_message_statuses(
        Some(sqlite_state_path.as_str()),
        &[created_message_id.clone()],
    )
    .expect("sqlite-backed relay projection should succeed");
    assert_eq!(projected_count, 1);

    let sqlite_backend = SqliteStoreBackend::open(Path::new(sqlite_state_path.as_str()))
        .expect("sqlite state file should be a valid sqlite database");
    let sqlite_payload = sqlite_backend
        .get(
            SERVICE_API_STATE_SQLITE_NAMESPACE,
            SERVICE_API_STATE_SQLITE_SNAPSHOT_KEY,
        )
        .expect("sqlite snapshot read should succeed")
        .expect("sqlite snapshot row should exist");
    let sqlite_json: Value =
        serde_json::from_slice(sqlite_payload.as_slice()).expect("sqlite snapshot should be json");
    assert_eq!(
        sqlite_json["messages"][created_message_id.as_str()]["status"],
        "relayed",
        "sqlite snapshot should persist projected created->relayed transition"
    );

    let mut reloaded = ServiceApiMessageStore::from_optional_state_file(Some(sqlite_state_path))
        .expect("sqlite-backed message store should reload");
    let reloaded_payload = reloaded
        .get_message_for_requester(created_message_id.as_str(), None)
        .expect("sqlite-backed message query should succeed")
        .expect("sqlite-backed message should exist");
    assert_eq!(
        reloaded_payload.status, "relayed",
        "sqlite-backed relay projection should promote created->relayed"
    );

    let _ = std::fs::remove_file(sqlite_state_file);
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

#[cfg(unix)]
#[test]
fn service_api_relay_spool_drain_errors_for_permission_denied_paths() {
    use std::os::unix::fs::PermissionsExt;

    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-spool-no-read-{unique_suffix}.ndjson"
    ));
    std::fs::write(spool_file.as_path(), "{\"message_id\":\"m1\"}\n")
        .expect("spool fixture should write");
    std::fs::set_permissions(spool_file.as_path(), std::fs::Permissions::from_mode(0o000))
        .expect("permissions should apply");

    let error = drain_service_api_relay_spool_entries(spool_file.to_str())
        .expect_err("permission denied spool file should fail open");
    assert!(
        error.contains("service api relay spool read failed"),
        "permission denied spool opens must fail closed"
    );

    let _ = std::fs::set_permissions(spool_file.as_path(), std::fs::Permissions::from_mode(0o600));
    let _ = std::fs::remove_file(spool_file);
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

#[test]
fn regression_message_store_refreshes_state_file_before_recipient_delivery_transition() {
    // Regression: #5917
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-message-store-refresh-{unique_suffix}.json"
    ));
    let state_path = state_file.to_string_lossy().to_string();
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-refresh-1":{
      "message_id":"msg-refresh-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender-refresh",
      "recipient_did":"kamn:did:agent:recipient-refresh",
      "body":"{\"message\":\"refresh\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("initial state fixture should write");

    let mut store = ServiceApiMessageStore::from_optional_state_file(Some(state_path))
        .expect("state-backed message store should initialize");

    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-refresh-1":{
      "message_id":"msg-refresh-1",
      "status":"relayed",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender-refresh",
      "recipient_did":"kamn:did:agent:recipient-refresh",
      "body":"{\"message\":\"refresh\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("relayed state fixture should write");

    let message_payload = store
        .get_message_for_requester("msg-refresh-1", Some("kamn:did:agent:recipient-refresh"))
        .expect("message query should succeed")
        .expect("message payload should exist");
    assert_eq!(
        message_payload.status, "delivered",
        "recipient query should observe daemon-projected relayed state and promote to delivered"
    );

    let persisted =
        std::fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value = serde_json::from_str(persisted.as_str()).expect("state json parses");
    assert_eq!(
        state_json["messages"]["msg-refresh-1"]["status"], "delivered",
        "delivery transition should persist after disk refresh"
    );

    let _ = std::fs::remove_file(state_file);
}

#[test]
fn integration_message_store_persists_data_layer_runtime_evidence_for_m0_to_m11() {
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-data-layer-evidence-{unique_suffix}.json"
    ));
    let state_path = state_file.to_string_lossy().to_string();

    let mut store = ServiceApiMessageStore::from_optional_state_file(Some(state_path.clone()))
        .expect("state-backed message store should initialize");
    let payload =
        r#"{"recipient_did":"kamn:did:agent:evidence-recipient","message":"wire-m0-m11"}"#;
    let created = store
        .create_message(
            payload,
            "api",
            Some("channel-evidence"),
            Some("kamn:did:agent:evidence-sender"),
            Some("kamn:did:agent:evidence-recipient"),
        )
        .expect("message creation should persist");
    let persisted =
        std::fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value = serde_json::from_str(persisted.as_str()).expect("state json parses");
    let evidence =
        &state_json["messages"][created.message_id.as_str()]["data_layer_runtime_evidence"];
    assert_eq!(
        evidence["schema_version"],
        "kamn.runtime.service-api-data-layer-runtime-evidence.v1"
    );
    assert!(
        evidence["m0_content_hash"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:")),
        "m0 evidence should include a sha256 content hash marker"
    );
    assert!(
        evidence["m1_merkle_root"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:")),
        "m1 evidence should include a merkle root"
    );
    assert!(
        evidence["m2_authorization_reason_code"].as_str().is_some(),
        "m2 evidence reason code should be persisted"
    );
    assert!(
        evidence["m3_blind_index_token"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:")),
        "m3 evidence should include blind-index token"
    );
    assert!(
        evidence["m4_transition_reason_code"].as_str().is_some(),
        "m4 evidence reason code should be persisted"
    );
    assert!(
        evidence["m5_record_hash"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:")),
        "m5 evidence should include record hash"
    );
    assert_eq!(
        evidence["m6_projection_edge_count"].as_u64(),
        Some(1),
        "m6 graph projection should include one edge"
    );
    assert!(
        evidence["m7_observability_health"].as_str().is_some(),
        "m7 observability health should be persisted"
    );
    assert!(
        evidence["m8_retention_due_count"].as_u64().is_some(),
        "m8 retention projection count should be persisted"
    );
    assert!(
        evidence["m9_dispatch_reason_code"].as_str().is_some(),
        "m9 dispatch reason code should be persisted"
    );
    assert!(
        evidence["m10_archived_partition_count"].as_u64().is_some(),
        "m10 archived partition count should be persisted"
    );
    assert!(
        evidence["m11_decision"].as_str().is_some(),
        "m11 closure decision should be persisted"
    );

    let _ = std::fs::remove_file(state_file);
}
