# Spec: Issue #5859 - Durable-by-Default Service API State Persistence

- Issue: #5859
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`kamn-node` Service API persistence only activates when `KAMN_SERVICE_API_STATE_FILE` is explicitly set. When unset, the endpoint falls back to in-memory state and loses data across restart. This keeps API->persistence integration fragile in default execution paths.

## Scope
In scope:
- Resolve a deterministic default state-file path when `KAMN_SERVICE_API_STATE_FILE` is not set.
- Keep explicit `KAMN_SERVICE_API_STATE_FILE` override behavior unchanged.
- Add restart integration coverage proving message state survives restart under default configuration.

Out of scope:
- Postgres-backed endpoint persistence.
- Daemon relay/consensus propagation.
- API payload schema changes.

## Acceptance Criteria
- AC-1: Service API resolves a deterministic default on-disk state-file path when `KAMN_SERVICE_API_STATE_FILE` is missing.
- AC-2: Message state created through `POST /v1/messages/send` is readable after process restart via `GET /v1/messages/{id}` without setting `KAMN_SERVICE_API_STATE_FILE`.
- AC-3: When `KAMN_SERVICE_API_STATE_FILE` is provided, explicit path precedence remains unchanged.
- AC-4: Targeted unit/integration/regression checks for the new persistence behavior pass.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | role + bind address, no `KAMN_SERVICE_API_STATE_FILE` | deterministic default state file path is produced |
| C-02 | AC-2 | Integration | send message -> stop server -> restart server -> query message, no state env override | same `message_id` and persisted payload available after restart |
| C-03 | AC-3 | Unit/Regression | explicit `KAMN_SERVICE_API_STATE_FILE=/tmp/custom.json` | runtime uses explicit path instead of derived default |
| C-04 | AC-4 | Verify | targeted `cargo test`/`fmt`/`clippy` for touched surfaces | all pass |

## Test Mapping
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env -- --exact`
- `cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_persists_message_state_across_restart -- --exact`
- `cargo test -p kamn-node service_api_endpoint::server::tests::unit_service_api_state_file_resolution_prefers_explicit_env_override -- --exact`
- `cargo test -p kamn-node service_api_endpoint::server::tests::unit_service_api_state_file_resolution_derives_deterministic_default_path_when_env_missing -- --exact`

## Success Metrics / Observable Signals
- Default (no state-file env) runtime no longer loses Service API message state across restart.
- Existing explicit-state-file persistence tests remain green.
- No regression in existing Service API route behavior.
