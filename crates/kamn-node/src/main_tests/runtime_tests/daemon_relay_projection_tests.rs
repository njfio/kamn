#[test]
fn integration_runtime_daemon_without_route_map_preserves_relay_spool_entries() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let bind_addr = reserve_runtime_test_loopback_addr();
    let state_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_state_file_path_for_bind_addr(
            bind_addr.as_str(),
        ),
    );
    let relay_spool_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_relay_spool_file_path_from_state_file(
            state_file.to_string_lossy().as_ref(),
        ),
    );
    std::fs::write(
        relay_spool_file.as_path(),
        concat!(
            "{\"message_id\":\"msg-relay-1\",\"sender_did\":\"kamn:did:agent:sender\",\"recipient_did\":\"kamn:did:agent:recipient\",\"body\":\"{\\\"message\\\":\\\"hello\\\"}\",\"queued_at_unix\":1700000001}\n",
            "{\"message_id\":\"msg-relay-2\",\"sender_did\":\"kamn:did:agent:sender\",\"recipient_did\":\"kamn:did:agent:recipient\",\"body\":\"{\\\"message\\\":\\\"world\\\"}\",\"queued_at_unix\":1700000002}\n"
        ),
    )
    .expect("relay spool fixture should write");

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--api-bind".to_owned(),
        bind_addr,
        "--daemon-max-ticks".to_owned(),
        "2".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon runtime should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    assert!(
        report.daemon_service_api_relay_drained_count.is_some(),
        "daemon runtime output must expose drained relay entry counters"
    );
    assert!(
        report.daemon_service_api_relay_projected_state_count.is_some(),
        "daemon runtime output must expose projected relay state counters"
    );
    assert!(
        report.daemon_observability_throughput_tps.unwrap_or(0) > 0,
        "daemon observability throughput should reflect projected relay work"
    );
    assert!(
        report.daemon_observability_latency_p50_ms.unwrap_or(0) > 0,
        "daemon observability latency should reflect measured tick processing"
    );

    let relay_contents = std::fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool should remain readable after daemon execution");
    assert!(
        relay_contents.contains("msg-relay-1") && relay_contents.contains("msg-relay-2"),
        "daemon runtime must preserve relay spool entries until a forwarding route is configured"
    );
    let _ = std::fs::remove_file(relay_spool_file);
    let _ = std::fs::remove_file(state_file);
}

#[test]
fn regression_runtime_daemon_without_route_map_keeps_created_message_pending() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let bind_addr = reserve_runtime_test_loopback_addr();
    let state_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_state_file_path_for_bind_addr(
            bind_addr.as_str(),
        ),
    );
    let relay_spool_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_relay_spool_file_path_from_state_file(
            state_file.to_string_lossy().as_ref(),
        ),
    );
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-relay-project-1":{
      "message_id":"msg-relay-project-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"hello\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state file fixture should write");
    std::fs::write(
        relay_spool_file.as_path(),
        r#"{"message_id":"msg-relay-project-1","sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"hello\"}","queued_at_unix":1700000003}
"#,
    )
    .expect("relay spool fixture should write");

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--api-bind".to_owned(),
        bind_addr,
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon runtime should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    assert!(
        report.daemon_service_api_relay_drained_count.is_some(),
        "daemon runtime output must expose drained relay entry counters for state projection"
    );
    assert!(
        report.daemon_service_api_relay_projected_state_count.is_some(),
        "daemon runtime output must expose projected relay state counters for state projection"
    );
    assert!(
        report.daemon_observability_throughput_tps.unwrap_or(0) > 0,
        "daemon observability throughput should reflect delayed relay processing work"
    );
    assert!(
        report.daemon_observability_latency_p50_ms.unwrap_or(0) > 0,
        "daemon observability latency should reflect measured delayed tick processing"
    );

    let state_payload = std::fs::read_to_string(state_file.as_path())
        .expect("state file should remain readable after daemon execution");
    let state_json: serde_json::Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["messages"]["msg-relay-project-1"]["status"],
        "created",
        "daemon relay processing without routing must not project created status"
    );
    let relay_payload = std::fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool should remain readable after daemon execution");
    assert!(
        relay_payload.contains("msg-relay-project-1"),
        "pending relay entry must remain queued for retry"
    );

    let _ = std::fs::remove_file(state_file);
    let _ = std::fs::remove_file(relay_spool_file);
}

#[test]
fn regression_runtime_daemon_relay_spool_drain_is_idempotent_when_empty() {
    // Regression: #5861
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-runtime-daemon-empty-relay-spool-{}-{}.ndjson",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    std::fs::write(relay_spool_file.as_path(), "").expect("empty relay spool fixture should write");
    let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
    let _relay_spool_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(relay_spool_file_str.as_str()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon runtime should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    let relay_contents = std::fs::read_to_string(relay_spool_file.as_path())
        .expect("empty relay spool should remain readable after daemon execution");
    assert!(
        relay_contents.is_empty(),
        "daemon runtime empty-drain path should preserve empty spool state"
    );
    let _ = std::fs::remove_file(relay_spool_file);
}

#[test]
fn regression_runtime_daemon_relay_state_projection_is_idempotent_for_relayed_messages() {
    // Regression: #5863
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-runtime-daemon-relayed-idempotent-state-{unique_suffix}.json"
    ));
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-runtime-daemon-relayed-idempotent-spool-{unique_suffix}.ndjson"
    ));
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-relayed-stable-1":{
      "message_id":"msg-relayed-stable-1",
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
    .expect("state file fixture should write");
    std::fs::write(
        relay_spool_file.as_path(),
        r#"{"message_id":"msg-relayed-stable-1","sender_did":"kamn:did:agent:sender","recipient_did":"kamn:did:agent:recipient","body":"{\"message\":\"stable\"}","queued_at_unix":1700000004}
"#,
    )
    .expect("relay spool fixture should write");
    let state_file_str = state_file.to_string_lossy().to_string();
    let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(&state_file_str));
    let _relay_spool_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(relay_spool_file_str.as_str()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "1".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon runtime should succeed");
    assert_eq!(report.runtime_mode, "daemon");

    let state_payload = std::fs::read_to_string(state_file.as_path())
        .expect("state file should remain readable after daemon execution");
    let state_json: serde_json::Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["messages"]["msg-relayed-stable-1"]["status"],
        "relayed",
        "projection should remain idempotent for already-relayed messages"
    );

    let _ = std::fs::remove_file(state_file);
    let _ = std::fs::remove_file(relay_spool_file);
}

#[test]
fn integration_runtime_daemon_without_route_map_requeues_relay_entries_arriving_during_tick_loop() {
    let _lock = log_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let bind_addr = reserve_runtime_test_loopback_addr();
    let state_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_state_file_path_for_bind_addr(
            bind_addr.as_str(),
        ),
    );
    let relay_spool_file = std::path::PathBuf::from(
        crate::service_api_endpoint::default_service_api_relay_spool_file_path_from_state_file(
            state_file.to_string_lossy().as_ref(),
        ),
    );
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-live-tick-1":{
      "message_id":"msg-live-tick-1",
      "status":"created",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender",
      "recipient_did":"kamn:did:agent:recipient",
      "body":"{\"message\":\"late-arrival\"}"
    }
  },
  "channel_messages":{},
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state file fixture should write");
    std::fs::write(relay_spool_file.as_path(), "").expect("relay spool fixture should write");

    let writer_spool_path = relay_spool_file.clone();
    let relay_writer = std::thread::spawn(move || {
        use std::io::Write;

        std::thread::sleep(std::time::Duration::from_millis(100));
        let mut relay_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(writer_spool_path.as_path())
            .expect("relay spool should open for delayed append");
        writeln!(
            relay_file,
            "{{\"message_id\":\"msg-live-tick-1\",\"sender_did\":\"kamn:did:agent:sender\",\"recipient_did\":\"kamn:did:agent:recipient\",\"body\":\"{{\\\"message\\\":\\\"late-arrival\\\"}}\",\"queued_at_unix\":1700000111}}"
        )
        .expect("relay spool delayed append should succeed");
    });

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--api-bind".to_owned(),
        bind_addr,
        "--daemon-max-ticks".to_owned(),
        "40".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "10".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon runtime should succeed");
    assert_eq!(report.runtime_mode, "daemon");
    assert!(
        report.daemon_service_api_relay_drained_count.is_some(),
        "daemon runtime output must expose drained relay entry counters for delayed arrivals"
    );
    assert!(
        report.daemon_service_api_relay_projected_state_count.is_some(),
        "daemon runtime output must expose projected relay state counters for delayed arrivals"
    );
    relay_writer
        .join()
        .expect("relay append worker should complete successfully");

    let state_payload = std::fs::read_to_string(state_file.as_path())
        .expect("state file should remain readable after daemon execution");
    let state_json: serde_json::Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["messages"]["msg-live-tick-1"]["status"],
        "created",
        "daemon tick loop must keep delayed relay entries pending until routing is configured"
    );

    let relay_payload = std::fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool should remain readable after daemon execution");
    assert!(
        relay_payload.contains("msg-live-tick-1"),
        "daemon tick loop must requeue delayed relay entries when forwarding is unavailable"
    );

    let _ = std::fs::remove_file(state_file);
    let _ = std::fs::remove_file(relay_spool_file);
}
