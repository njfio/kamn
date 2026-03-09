use super::super::*;
use super::support::{
    build_message_snapshot, read_state_json, send_persisted_message, unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-data-layer-evidence-state");
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let snapshot = build_message_snapshot("127.0.0.1:34082");
    let bind_addr = reserve_loopback_addr();
    let send_payload = send_persisted_message(
        &snapshot,
        bind_addr.as_str(),
        "kamn:did:agent:e2e-sender",
        81,
        r#"{"recipient_did":"kamn:did:agent:e2e-recipient","message":"e2e-runtime-evidence"}"#,
    );
    assert_runtime_evidence(
        read_state_json(state_file.as_path())["messages"][send_payload.message_id.as_str()]
            ["data_layer_runtime_evidence"]
            .to_owned(),
    );
    let _ = fs::remove_file(state_file);
}

fn assert_runtime_evidence(evidence: Value) {
    assert_eq!(
        evidence["schema_version"],
        "kamn.runtime.service-api-data-layer-runtime-evidence.v1"
    );
    assert_hash_fields(&evidence);
    for key in [
        "m2_authorization_reason_code",
        "m3_blind_index_token",
        "m4_transition_reason_code",
        "m5_record_hash",
        "m7_observability_health",
        "m9_dispatch_reason_code",
        "m11_decision",
    ] {
        assert!(evidence[key].as_str().is_some());
    }
    for key in [
        "m6_projection_edge_count",
        "m8_retention_due_count",
        "m10_archived_partition_count",
    ] {
        assert!(evidence[key].as_u64().is_some());
    }
}

fn assert_hash_fields(evidence: &Value) {
    for key in ["m0_content_hash", "m1_merkle_root"] {
        assert!(evidence[key]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:")));
    }
}
