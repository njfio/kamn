# Spec - Issue #4124

- Title: Subtask: add red tests for observability sample projection into metrics and health surfaces
- Parent: #4118
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

Observability sample structures must project deterministically into metrics and health surfaces; missing markers must fail closed.

## Objective

Provide deterministic red-to-green projection coverage for observability markers across metrics and health endpoints.

## Scope

In scope:
- Projection drift/fail checks for local observability scrape policy.
- Projection drift/fail checks for service API prometheus metrics policy.

Out of scope:
- External telemetry sink forwarding.

## Acceptance Criteria

- AC-1: Projection drift in observability scrape surfaces fails closed.
- AC-2: Projection drift in prometheus metrics surfaces fails closed.
- AC-3: Baseline projection parity passes deterministically.

## Conformance Cases

- C-01 (AC-1): `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh` passes.
- C-02 (AC-2): `bash scripts/runtime/test_check_service_api_prometheus_metrics_live_policy.sh` passes.
- C-03 (AC-3): baseline pass paths in both suites remain deterministic.

## Success Metrics

- Projection marker regressions are caught deterministically by runtime policy suites.
