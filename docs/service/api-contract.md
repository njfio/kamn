# Service API Contract

## Scope

This contract defines deterministic fail-closed taxonomy markers for async lifecycle limiter rejection projection in Task #4311 and Subtask #4316.

## Async Lifecycle Rejection Taxonomy (Issue #4316)

- `service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1`
- `service_api_lifecycle_rejection_reason_codes_csv=service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid`

## Async Lifecycle Rejection Projection Matrix

| Reason code | Rejection class | HTTP status | Error label | Outcome |
|---|---|---|---|---|
| `service_api_ingress_concurrency_limit_exceeded` | `async-lifecycle-limiter` | `429` | `too-many-requests` | `concurrency-limit` |
| `service_api_ingress_rate_limit_exceeded` | `async-lifecycle-limiter` | `429` | `too-many-requests` | `rate-limit` |
| `service_api_ingress_sender_rate_limit_exceeded` | `sender-admission-limiter` | `429` | `too-many-requests` | `anti-spam` |
| `service_api_ingress_sender_suspended` | `sender-admission-limiter` | `429` | `too-many-requests` | `anti-spam` |
| `service_api_ingress_sender_duplicate_message_id` | `sender-admission-limiter` | `429` | `too-many-requests` | `anti-spam` |
| `service_api_ingress_sender_insufficient_deposit` | `sender-admission-limiter` | `429` | `too-many-requests` | `anti-spam` |
| `service_api_ingress_anti_spam_engine_invalid` | `async-lifecycle-engine` | `500` | `internal` | `anti-spam-error` |

## Validation Commands

- `cargo test -p kamn-node lifecycle_projection_ -- --nocapture`
- `cargo test -p kamn-node lifecycle_rejection_projection -- --nocapture`
- `cargo test -p kamn-core --test service_api_lifecycle_contract_docs`

## Regression

- Async lifecycle limiter rejection projection remains fail-closed and deterministic (`Regression: #4316`).
