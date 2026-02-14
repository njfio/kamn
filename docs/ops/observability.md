# Runtime Observability (Phase 6.1)

This document captures the current production-service observability surface for roadmap
Story #2960 and Task #2961.

## Endpoints

- Service API metrics: `GET /metrics`
- Service API health: `GET /healthz`
- Optional standalone observability endpoint:
  - `--observability-endpoint-bind <host:port>`
  - `--observability-endpoint-metrics-path </path>` (default `/metrics`)
  - `--observability-endpoint-health-path </path>` (default `/healthz`)

## Prometheus Metrics Contract

Service API `/metrics` exports deterministic Prometheus text format metrics:

- `kamn_service_api_health{runtime_mode="<mode>"}`
- `kamn_service_api_role{role="<role>"}`
- `kamn_service_api_chain_info{chain_id="<id>",chain_version="<version>"}`
- `kamn_service_api_observability_latency_p50_ms`
- `kamn_service_api_observability_latency_p99_ms`
- `kamn_service_api_observability_throughput_tps`
- `kamn_service_api_observability_error_rate_bps`
- `kamn_service_api_observability_availability_bps`
- `kamn_service_api_observability_alert_count`
- `kamn_service_api_observability_source{source="<daemon|kolme-live|unknown>"}`
- `kamn_service_api_observability_health{health="<healthy|degraded|critical|unknown>"}`

When daemon/kolme runtime telemetry is unavailable, the export fails closed to deterministic
unknown defaults (`source=unknown`, health value `0`, numeric gauges `0`) instead of omitting
the metric families.

## Structured Telemetry

- Node runtime logs support machine-parsable JSON output through `KAMN_NODE_LOG_FORMAT=json`.
- Runtime telemetry values are mapped from daemon or kolme-live execution snapshots and exposed
  through the service metrics contract above.
- Health payload includes telemetry source and health classification:
  - `{"status":"ok","runtime_mode":"...","role":"...","observability_source":"...","observability_health":"..."}`

## Local Validation

Low-cost local checks:

- `cargo test -p kamn-node service_api_endpoint -- --nocapture`
- `cargo test -p kamn-node unit_service_api_endpoint_metrics_use_runtime_observability_when_present -- --nocapture`
- `cargo fmt --check`
- `cargo clippy -p kamn-node -- -D warnings`
- `bash scripts/runtime/validate_service_api_live.sh`

These checks validate metrics export behavior without introducing external metrics backends or
long-running CI load.
