# Spec - Issue #4118

- Title: Task: wire observability structures to emitted metrics and health projections
- Parent: #4115
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

Observability sample/alert structures required deterministic wiring into emitted metrics and health surfaces with explicit marker parity.

## Objective

Close observability emission wiring task with deterministic runtime policy/contract validation across projection and emission lanes.

## Scope

In scope:
- Projection drift and parity checks (`#4124`).
- Deterministic emission lane behavior (`#4125`).
- Task-level closure artifacts and conformance mapping.

Out of scope:
- External telemetry sink configuration.

## Acceptance Criteria

- AC-1: Metrics and health projections consume observability structures deterministically.
- AC-2: Projection and emission marker parity is fail-closed on drift.
- AC-3: Runtime emission contract suites pass deterministically.
- AC-4: Task-level conformance remains auditable and green.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/runtime/test_check_local_observability_scrape_live_policy.sh` and `bash scripts/runtime/test_check_service_api_prometheus_metrics_live_policy.sh` pass.
- C-02 (AC-3): `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh` and `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh` pass.
- C-03 (AC-4): consolidated checks above pass in task closure run.

## Success Metrics

- Observability projection/emission behavior is deterministic and protected by fail-closed contracts.
