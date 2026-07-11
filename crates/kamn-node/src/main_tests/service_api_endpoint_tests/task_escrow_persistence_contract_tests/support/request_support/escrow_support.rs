use super::*;

pub(crate) fn fund_escrow(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    payload: &str,
) -> Value {
    let canonical_payload = canonical_escrow_payload(caller_did, nonce, payload);
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: "/v1/escrow/fund",
            caller_did,
            nonce,
            body: canonical_payload.as_str(),
            extra_headers: &[],
        },
    );
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("escrow fund payload should deserialize")
}

pub(crate) fn release_escrow(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    escrow_id: &str,
) -> Value {
    let response = release_escrow_response(snapshot, bind_addr, caller_did, nonce, escrow_id);
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(response.as_str()))
        .expect("escrow release payload should deserialize")
}

pub(crate) fn release_escrow_response(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    caller_did: &str,
    nonce: u64,
    escrow_id: &str,
) -> String {
    let body = format!(r#"{{"idempotency_key":"escrow-release-{nonce}"}}"#);
    let response = signed_request(
        snapshot,
        bind_addr,
        SignedRequest {
            max_requests: 1,
            method: "POST",
            path: format!("/v1/escrow/{escrow_id}/release").as_str(),
            caller_did,
            nonce,
            body: body.as_str(),
            extra_headers: &[],
        },
    );
    response
}

fn canonical_escrow_payload(caller_did: &str, nonce: u64, payload: &str) -> String {
    if payload.contains("\"transaction_id\"") {
        return payload.to_owned();
    }
    let input: Value = serde_json::from_str(payload).expect("legacy escrow fixture should parse");
    let task_id = input["task_id"]
        .as_str()
        .expect("task id should be present");
    let state_file = std::env::var("KAMN_SERVICE_API_STATE_FILE").expect("state file env");
    let state = super::super::env_support::read_state_json(std::path::Path::new(&state_file));
    canonical_payload(caller_did, nonce, &input, task_id, &state["tasks"][task_id])
}

fn canonical_payload(
    caller_did: &str,
    nonce: u64,
    input: &Value,
    task_id: &str,
    task: &Value,
) -> String {
    let amount = input["amount_lamports"]
        .as_u64()
        .or_else(|| input["amount"].as_u64())
        .unwrap_or(1);
    serde_json::json!({
        "task_id": task_id,
        "transaction_id": task["transaction_id"],
        "beneficiary_did": task["provider_did"],
        "amount_lamports": amount,
        "network": "solana-devnet",
        "terms_digest": task["terms_digest"],
        "release_authority_did": test_service_api_sender_did(caller_did),
        "release_policy": "task-completed",
        "idempotency_key": format!("escrow-fund-{nonce}"),
    })
    .to_string()
}
