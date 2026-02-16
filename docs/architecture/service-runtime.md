# Service Runtime Shutdown Contracts

This document defines deterministic shutdown marker semantics for daemon and
full runtime execution paths.

## Scope

- Runtime mode: `daemon`, `full`
- Trigger sources:
- deterministic signal-tick controls (`--daemon-shutdown-signal-tick`)
- OS signals on Unix (default when no explicit signal-tick controls are provided)
- explicit OS-signal override flag (`--daemon-shutdown-os-signals`) remains
  supported for policy clarity and cross-platform fail-closed checks

## Runtime Phase Module Map

- `crates/kamn-node/src/runtime_orchestration.rs`
  - owns runtime mode dispatch, production transport profile policy checks, full
    supervisor stop-contract validation, and signer policy enforcement.
- `crates/kamn-node/src/runtime_orchestration/daemon_phase.rs`
  - owns daemon/full daemon-phase execution, peer lifecycle transition
    projection, and shutdown marker parsing/projection.
- Extraction contract lane:
  - `cargo test -p kamn-node --test main_module_extraction_contract`

## Drain Marker Contract

`shutdown_drain_status` is emitted on daemon completion and full-supervisor stop
markers with these values:

- `completed`: shutdown drain target completed before timeout budget.
- `timeout`: shutdown drain exceeded timeout budget and failed closed.
- `not-signaled`: no shutdown signal triggered during runtime tick budget.

## Snapshot Flush Marker Contract

`shutdown_snapshot_flush_status` is emitted on daemon completion and full
supervisor stop markers with these values:

- `snapshot-flushed`: graceful shutdown path committed final snapshot flush.
- `snapshot-flush-timeout`: timeout shutdown path emitted forced final flush
  marker and failed closed.
- `snapshot-not-requested`: no signal-triggered shutdown occurred.

## Fail-Closed Validation

The full-supervisor stop contract validator enforces deterministic consistency
between completion reason, drain status, and snapshot flush status:

- Unknown status values are rejected.
- `tick-budget-exhausted` requires `not-signaled` + `snapshot-not-requested`.
- `graceful-shutdown:*` requires `completed` + `snapshot-flushed`.
- `graceful-shutdown-timeout:*` requires `timeout` + `snapshot-flush-timeout`.
- signal-tick controls, when present, take precedence over default OS-signal
  shutdown policy.

Invalid combinations emit deterministic reason codes and fail closed.

## Local Retry/Diagnostics Governance Contract

Local retry/backoff diagnostics are enforced through a deterministic summary +
policy + contract-lane chain:

- summary lane:
  - `scripts/runtime/validate_local_retry_diagnostics_live.sh`
- policy checker:
  - `scripts/runtime/check_local_retry_diagnostics_live_policy.sh`
- composed lane:
  - `scripts/runtime/validate_local_retry_diagnostics_live_contract_lane.sh`

Deterministic retry governance markers:

- `retry_readiness_status=verified`
- `retry_backoff_status=verified`
- `retry_jitter_parity_status=verified`
- `reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v1`
- `reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,ci_local_network_budget_boundary_exceeded`

Fail-closed drift/budget markers:

- `local_retry_readiness_progress_stalled`
- `local_retry_backoff_jitter_parity_bypass_detected`
- `ci_local_network_budget_boundary_exceeded`

CI-local budget is explicitly bounded and fail-closed:

- contract lane rejects `--max-seconds > 240`.
- run-mode local-heavy checks remain opt-in and excluded from fast-gate paths.

## Observability Route Topology Contract

Observability serving is on the unified axum route stack and no longer uses a
hand-rolled parser/listener path.

- Runtime path:
  - `serve_observability_endpoint(...)` (sync wrapper)
  - `serve_observability_endpoint_async(...)` (async runtime path)
  - `build_observability_endpoint_router(...)` (axum route composition)
- Route topology:
  - root route: `"/"` via `any(handle_observability_http_route)`
  - wildcard route: `"/{*path}"` via `any(handle_observability_http_route)`
- Fail-closed malformed request contracts:
  - non-`GET` methods resolve to deterministic `404 not found`
  - `GET` requests with non-empty body indicators (`Content-Length > 0` or
    `Transfer-Encoding`) resolve to deterministic `404 not found`
  - request budget and idle timeout remain deterministic and fail closed

### Observability Governance Taxonomy

Runtime observability endpoint policy checking enforces deterministic readiness +
stream parity governance markers:

- `endpoint_readiness_status=verified`
- `stream_parity_status=verified`
- `reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1`
- `reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded`

Fail-closed reason taxonomy includes:

- `runtime_observability_endpoint_readiness_progress_stalled`
- `runtime_observability_stream_parity_bypass_detected`
- `ci_local_observability_endpoint_budget_boundary_exceeded`

CI-local boundary remains explicit and fail-closed:

- observability endpoint contract lane rejects `--max-seconds > 240`.

## Service API + Observability Route Compatibility Matrix Contract

Compatibility between service API routes and observability routes is enforced by
a deterministic matrix lane with explicit route/method/status/content-type
expectations.

- lane entry:
  - `scripts/runtime/validate_service_api_observability_route_compatibility_live.sh`
- policy checker:
  - `scripts/runtime/check_service_api_observability_route_compatibility_live_policy.sh`
- composed contract lane:
  - `scripts/runtime/validate_service_api_observability_route_compatibility_live_contract_lane.sh`

Matrix row coverage includes:

- service API:
  - `GET /healthz -> 200 application/json`
  - `GET /metrics -> 200 text/plain`
  - `POST /metrics -> 405 application/json`
- observability endpoint:
  - `GET /metrics -> 200 text/plain`
  - `GET /healthz -> 200 application/json`
  - `GET /unknown -> 404 application/json`

Fail-closed drift taxonomy includes:

- `service_api_observability_route_compatibility_policy_matrix_row_missing:<row_id>`
- `service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:<row_id>`
- `service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch:<row_id>`

## Combined Native Transport + Kolme Commit Validation Flow

The local full-stack integration lane composes native libp2p transport and
Kolme runtime-commit checks in one local-heavy run.

- Entry lane:
  - `scripts/runtime/validate_local_full_stack_integration_live.sh`
- Composed run-mode lanes:
  - `scripts/runtime/validate_full_io_scenario_matrix_live.sh`
  - `scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh`
  - `scripts/runtime/check_libp2p_convergence_process_isolated_live_policy.sh`
  - `scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh`
  - `scripts/kolme/check_local_kamn_live_runtime_integration_policy.py`
- Combined fail-closed taxonomy contracts:
  - `combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1`
  - `combined_transport_reason_codes=fork_choice_stale_block_height`
  - `combined_kolme_runtime_reason_code`
  - `kolme_runtime_commit_failure_taxonomy_version=v1`
  - `kolme_runtime_commit_failure_taxonomy`
  - `kolme_fixture_profile=real-node-non-synthetic-v1`
  - `kolme_fixture_profile_version=v1`
  - `kolme_fixture_profile_status`
