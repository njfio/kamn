use super::super::*;
use super::support::{
    build_directory_snapshot, query_agent_profile, raw_signed_request, read_state_json,
    unique_named_state_file,
};

#[test]
fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-agent-profile-restart-state");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_text.as_str()));
    let caller_did = "kamn:did:agent:test-client-agent-profile-restart";
    let target_agent_did = "kamn:did:agent:profile-restart-target";

    let first_snapshot = build_directory_snapshot("127.0.0.1:34119");
    let first_profile = query_agent_profile(&first_snapshot, reserve_loopback_addr().as_str(), caller_did, 121, target_agent_did);
    let phase_one_state_json = read_state_json(state_file.as_path());
    assert_eq!(first_profile.did, target_agent_did);
    assert_eq!(first_profile.reputation_score, 500);
    assert_eq!(phase_one_state_json["agents"][target_agent_did]["did"], target_agent_did);
    assert_eq!(phase_one_state_json["agents"][target_agent_did]["reputation_score"], 500);

    let restart_snapshot = build_directory_snapshot("127.0.0.1:34120");
    let restart_profile = query_agent_profile(
        &restart_snapshot,
        reserve_loopback_addr().as_str(),
        caller_did,
        122,
        target_agent_did,
    );
    assert_eq!(restart_profile.did, target_agent_did);
    assert_eq!(restart_profile.reputation_score, 500);
    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-agent-profile-legacy-path");
    let state_file_text = state_file.to_string_lossy().to_string();
    let _state_file_guard = EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_text.as_str()));
    let snapshot = build_directory_snapshot("127.0.0.1:34121");
    let caller_did = "kamn:did:agent:test-client-agent-profile-legacy-path";
    let legacy_target_did = "did:kamn:agent:legacy-alpha";
    let response = raw_signed_request(
        &snapshot,
        reserve_loopback_addr().as_str(),
        1,
        "GET",
        format!("/v1/agents/{legacy_target_did}").as_str(),
        caller_did,
        121,
        "",
        &[],
    );

    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(error_payload.reason_code, SERVICE_API_AGENT_DID_PATH_INVALID_REASON_CODE);
    assert!(error_payload.message.contains("invalid agent did path"));

    if state_file.exists() {
        let state_json = read_state_json(state_file.as_path());
        assert!(state_json["agents"].get(legacy_target_did).is_none());
    }
    let _ = fs::remove_file(state_file);
}
