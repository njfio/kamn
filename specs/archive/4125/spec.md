# Spec - Issue #4125

- Title: Subtask: implement deterministic metrics healthz and stream emission wiring from observability structures
- Parent: #4118
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

Operational metrics/health/stream emission wiring must be deterministic and policy-validated across contract lanes.

## Objective

Validate deterministic emission wiring from observability structures into metrics and health contract lanes.

## Scope

In scope:
- Local observability scrape contract lane emission checks.
- Service API prometheus metrics contract lane emission checks.

Out of scope:
- External dashboard sink behavior.

## Acceptance Criteria

- AC-1: Local observability scrape emission lane remains deterministic and verified.
- AC-2: Service API prometheus emission lane remains deterministic and verified.
- AC-3: Emission-policy marker mapping remains fail closed on drift.

## Conformance Cases

- C-01 (AC-1): `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh` passes.
- C-02 (AC-2/AC-3): `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh` passes.

## Success Metrics

- Emission wiring remains deterministic and protected by contract-lane drift checks.
