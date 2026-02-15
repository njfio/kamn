# Streaming Observability Contracts

## Scope

This document defines the deterministic stream telemetry contract for `kamn-node`
runtime observability and the bounded local validation policy used in CI.

## Stream Payload Contract

- Endpoint: `GET /metrics.stream`
- Content type: `application/x-ndjson`
- Schema marker: `schema_version="kamn.runtime.observability.stream.v1"`
- Deterministic fields:
  - `source`
  - `runtime_mode`
  - `health`
  - `reason_code`
  - `ready`
  - `readiness_reason_code`
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

## Backpressure and Reconnect Contracts

- Queue/backpressure behavior remains bounded by request-budget controls:
  - `--observability-endpoint-max-requests`
  - `--observability-endpoint-idle-timeout-ms`
- Local validation lane emits deterministic contract markers:
  - `stream_reconnect_churn_status=verified`
  - `queue_bound_budget_status=verified`
  - `readiness_failure_drill_status=verified`
  - `scrape_failure_taxonomy_status=verified`
  - `scrape_failure_taxonomy_csv=readiness_failure_drill_status,stream_reconnect_churn_status,queue_bound_budget_status`
- Readiness reason taxonomy is deterministic:
  - `none`
  - `readiness_transport_dependency_unhealthy`
  - `readiness_signer_dependency_unhealthy`
  - `readiness_commit_dependency_unhealthy`
  - `readiness_runtime_health_degraded`

## Low-Cost Validation Lane

- `bash scripts/runtime/validate_local_observability_scrape_live.sh --mode dry-run --output-json /tmp/local-observability-scrape-live-summary.json`
- `bash scripts/runtime/check_local_observability_scrape_live_policy.sh --report-file /tmp/local-observability-scrape-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-observability-scrape-live-policy.json`
- `bash scripts/runtime/validate_local_observability_scrape_live_contract_lane.sh --output-json /tmp/local-observability-scrape-live-contract-lane-report.json --policy-output-json /tmp/local-observability-scrape-live-policy.json`

Regression: #3602
