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
    let state_json = read_state_json(state_file.as_path());
    let evidence =
        &state_json["messages"][send_payload.message_id.as_str()]["data_layer_runtime_evidence"];
    assert_eq!(
        evidence["schema_version"],
        "kamn.runtime.service-api-data-layer-runtime-evidence.v1"
    );
    assert!(evidence["m0_content_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(evidence["m1_merkle_root"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(evidence["m2_authorization_reason_code"].as_str().is_some());
    assert!(evidence["m3_blind_index_token"].as_str().is_some());
    assert!(evidence["m4_transition_reason_code"].as_str().is_some());
    assert!(evidence["m5_record_hash"].as_str().is_some());
    assert!(evidence["m6_projection_edge_count"].as_u64().is_some());
    assert!(evidence["m7_observability_health"].as_str().is_some());
    assert!(evidence["m8_retention_due_count"].as_u64().is_some());
    assert!(evidence["m9_dispatch_reason_code"].as_str().is_some());
    assert!(evidence["m10_archived_partition_count"].as_u64().is_some());
    assert!(evidence["m11_decision"].as_str().is_some());
    let _ = fs::remove_file(state_file);
}
