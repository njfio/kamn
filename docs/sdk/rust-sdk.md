# Rust SDK Service Client

This document covers the Rust SDK service client added for Service Phase 5 core delivery (Task #2946).

## Scope

The `kamn-sdk` crate now includes a synchronous service client for the runtime API contract:

- `ServiceApiClient`
- `ServiceRequestAuth`
- `service_signature_for_fields`
- typed response models for messages/channels/tasks/agent profile/health/events

The client targets deterministic service routes exposed by `kamn-node` runtime mode `api`:

- `POST /v1/messages/send`
- `GET /v1/messages/{id}`
- `POST /v1/channels/create`
- `POST /v1/tasks/create`
- `GET /v1/tasks/{id}`
- `GET /v1/agents/{did}`
- `GET /healthz`
- `GET /metrics`
- `GET /v1/events/ws` (upgrade + single event frame)

## Usage

```rust
use kamn_sdk::{
    service_signature_for_fields, AgentDid, ServiceApiClient, ServiceRequestAuth,
};

let client = ServiceApiClient::connect("http://127.0.0.1:34052")?;
let sender = AgentDid::parse("kamn:did:agent:alpha")?;

let body = r#"{"message":"hello"}"#;
let signature = service_signature_for_fields(&sender, 1, "kolme-localnet", "v0", body);
let auth = ServiceRequestAuth::new(sender.clone(), 1, signature)?;

let receipt = client.send_message(body, &auth)?;
let status = client.get_message(receipt.message_id.as_str(), &ServiceRequestAuth::new(
    sender.clone(),
    2,
    service_signature_for_fields(&sender, 2, "kolme-localnet", "v0", ""),
)?)?;

assert_eq!(status.status, "created");
```

## Validation

Core delivery validation for Task #2946:

- `cargo test -p kamn-sdk --test service_api_client`
- `cargo test -p kamn-sdk`
- `cargo clippy -p kamn-sdk -- -D warnings`
- `cargo fmt --check`

Live validation lane evidence is tracked separately in Task #2948 / Subtask #2949.
