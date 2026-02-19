# Observability Contracts

## Runtime Endpoint Schemas

This document defines deterministic payload contracts for the runtime observability endpoint.

- `GET /metrics` emits Prometheus text with readiness and reason-code labels.
- `GET /healthz` emits JSON with `schema_version="kamn.runtime.observability.health.v1"`.
- `GET /readyz` emits JSON with `schema_version="kamn.runtime.observability.readiness.v1"`.
- `GET /metrics.stream` emits NDJSON with `schema_version="kamn.runtime.observability.stream.v1"`.

## Route Parity Matrix Contract

- `observability_route_parity_matrix_version=kamn.runtime.observability.route-parity.v1`
- Baseline and secure-mode route parity matrix:
  - `GET /metrics -> 200 text/plain; version=0.0.4`
  - `GET /healthz -> 200 application/json`
  - `GET /readyz -> 200 application/json`
  - `GET /metrics.stream -> 200 application/x-ndjson`
- Fail-closed parity matrix rows:
  - `GET /unknown -> 404 text/plain; charset=utf-8`
  - `POST /metrics -> 404 text/plain; charset=utf-8`
- Deterministic parity checkpoint markers:
  - `route_parity_checkpoint_status=verified`
  - `fail_closed_checkpoint_status=verified`
  - `route_class_coverage_status=verified`
- Fail-closed parity drift taxonomy markers:
  - `service_api_observability_route_compatibility_policy_matrix_row_missing:<row_id>`
  - `service_api_observability_route_compatibility_policy_matrix_row_route_mismatch:<row_id>`
  - `service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:<row_id>`
  - `service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch:<row_id>`
  - `service_api_observability_route_compatibility_policy_marker_missing:route_parity_checkpoint_status`

## Tracing Event Taxonomy Contract

- `tracing_event_taxonomy_version=kamn.node.tracing-event-taxonomy.v1`
- Required event-field vocabulary:
  - `execution_id`
  - `runtime_mode`
  - `route`
  - `reason_code`
  - `transport_checkpoint_failures`
  - `signer_checkpoint_failures`
  - `commit_checkpoint_failures`
- Required runtime event markers:
  - `runtime_daemon_tick_summary`
  - `runtime_daemon_shutdown_checkpoint_reconciliation`
  - `runtime_observability_endpoint_request`
- Fail-closed drift reason markers:
  - `runtime_tracing_taxonomy_required_field_missing:<event>:<field>`
  - `runtime_tracing_taxonomy_schema_drift:<event>:<field>`
  - `runtime_tracing_taxonomy_event_marker_missing:<event>`

## Startup Logging Configuration Contract

- `startup_logging_configuration_version=kamn.node.startup-logging-config.v1`
- Runtime modes with deterministic tracing bootstrap:
  - `bootstrap`
  - `full`
  - `kolme-live`
- Environment controls:
  - `KAMN_NODE_LOG_LEVEL` accepted values: `error`, `warn`, `info`, `debug`, `trace`
  - `KAMN_NODE_LOG_FORMAT` accepted values: `text`, `json`
- Fail-closed invalid config markers:
  - `ConfigError::InvalidLogConfig`
  - `KAMN_NODE_LOG_LEVEL must be one of: error,warn,info,debug,trace`
  - `KAMN_NODE_LOG_FORMAT must be one of: text,json`

## Health Payload Contract

`/healthz` includes:

- `schema_version`
- `source`
- `runtime_mode`
- `health`
- `alert_count`
- `reason_code`
- `ready`
- `readiness_reason_code`
- `readiness_reason_taxonomy_version`
- `transport_dependency_status`
- `signer_dependency_status`
- `commit_dependency_status`
- `transport_checkpoint_failures`
- `signer_checkpoint_failures`
- `commit_checkpoint_failures`
- `latency_p50_ms`
- `latency_p99_ms`
- `throughput_tps`
- `error_rate_bps`
- `availability_bps`

## Readiness Payload Contract

`/readyz` includes:

- `schema_version`
- `source`
- `runtime_mode`
- `ready`
- `health`
- `reason_code`
- `readiness_reason_code`
- `readiness_reason_taxonomy_version`
- `transport_dependency_status`
- `signer_dependency_status`
- `commit_dependency_status`
- `transport_checkpoint_failures`
- `signer_checkpoint_failures`
- `commit_checkpoint_failures`

## Readiness Taxonomy Contract

- `readiness_reason_taxonomy_version="kamn.runtime.observability.readiness.reason-taxonomy.v1"`
- `none`
- `readiness_transport_dependency_unhealthy`
- `readiness_signer_dependency_unhealthy`
- `readiness_commit_dependency_unhealthy`
- `readiness_runtime_health_degraded`

Regression: #3600
