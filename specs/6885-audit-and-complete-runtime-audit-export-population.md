## Objective

Wire deterministic audit-export population into the real service API runtime so selected message and task flows emit persisted audit export bundles, and fail loudly when the export sidecar cannot be produced.

## Inputs/Outputs

Inputs:
- Existing audit export engine in `crates/kamn-core/src/audit_exports.rs`
- Existing service API runtime state and state-file path derivation in `crates/kamn-node/src/service_api_endpoint.rs`, `server.rs`, and `state_io.rs`
- Existing real message and task mutation routes in `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/**`
- Existing service API integration support under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/**`

Outputs:
- Deterministic audit export file path derived from the service API state file
- Runtime wiring that records and persists audit export bundles for selected real service API flows
- Integration coverage proving that a real service API request writes an audit export bundle
- Loud, observable failure when audit export persistence cannot be completed

## Boundaries/Non-goals

Non-goals:
- Redesigning `AuditExportEngine`
- Retrofitting every runtime path in one issue
- Replacing existing audit-view or compliance domain models
- Designing long-term multi-node audit aggregation

Boundaries:
- Keep code changes scoped to `crates/kamn-node/**`, `crates/kamn-core/tests/**`, and this spec
- Use the service API runtime as the first concrete export producer
- Limit new runtime wiring to selected message/task flows that already exist as real entrypoints

## Failure modes

- Audit export path derivation is missing or unstable and the runtime silently skips export
- Message or task runtime flow succeeds but audit export bundle is never written
- Existing export bundle cannot be loaded or parsed and the runtime silently resets or ignores it
- Export sidecar write/open/serialize failure is swallowed instead of surfacing as a hard failure
- Integration coverage exercises only helpers instead of a real service API request path

## Acceptance criteria

- [ ] Service API runtime derives a deterministic audit export sidecar path from the state file
- [ ] Selected real message/task mutation routes append audit export records into a persisted bundle
- [ ] Audit export persistence reuses `AuditExportEngine` instead of inventing a parallel format
- [ ] At least one integration test proves a real service API request creates or updates the audit export bundle
- [ ] Runtime returns a loud error when audit export persistence cannot be produced for the selected flows
- [ ] `cargo test -p kamn-node` targeted audit-export runtime coverage passes

## Files to touch

- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/create_routes.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/message_routes.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/**`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/**` or an equivalent real service API integration test target

## Error semantics

- Audit export path derivation errors must be returned as `Result<_, String>` startup failures where applicable
- Export bundle load/parse/write failures must return explicit `500` responses from the selected runtime routes
- No silent fallback to “export disabled” is allowed once a state-backed runtime path is active
- Existing message/task persistence errors must remain explicit and unchanged unless the route now fails because audit export persistence failed

## Test plan

Red:
- Add a contract or integration test that expects a deterministic audit export file path/bundle after a real service API message or task request
- Add an error-path test that forces audit export persistence failure and expects a loud `500` response or startup error

Green:
- Derive the audit export sidecar path from the existing service API state file
- Persist audit export bundles using `AuditExportEngine`
- Wire selected message/task runtime routes to record and persist export events

Refactor/Integration:
- Keep helper files and functions within active size policy
- Reuse existing service API state-file helpers and integration harnesses
- Re-run targeted service API integration coverage and touched-Rust policy
