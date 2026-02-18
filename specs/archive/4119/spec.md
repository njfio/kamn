# Spec - Issue #4119

- Title: Task: add observability emission contract lanes with low-cost ci smoke checks
- Parent: #4115
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

Observability emission governance requires deterministic marker-lineage checking in CI while preserving low-cost fast-gate boundaries.

## Objective

Close observability CI checker and local-heavy exclusion policy coverage with fail-closed drift detection.

## Scope

In scope:
- CI drift checker for observability endpoint marker lineage.
- CI policy guards ensuring local-heavy scrape lanes remain explicit opt-in.
- Task-level closure artifacts and conformance mapping.

Out of scope:
- Always-on heavy scrape lanes in PR CI.

## Acceptance Criteria

- AC-1: CI checker validates observability marker lineage deterministically.
- AC-2: Local-heavy scrape lanes remain explicit opt-in through CI exclusion policy.
- AC-3: Drift checks fail closed on missing/mismatched markers.
- AC-4: Unit/Functional/Integration/Regression coverage for this task remains green.

## Conformance Cases

- C-01 (AC-1/AC-3): `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_local_observability_scrape_ci_exclusion_policy.sh` passes.
- C-03 (AC-2): `bash scripts/ci/test_service_api_prometheus_metrics_ci_exclusion_policy.sh` passes.
- C-04 (AC-4): `bash scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh` and `bash scripts/runtime/test_validate_service_api_prometheus_metrics_live_contract_lane.sh` pass.

## Success Metrics

- CI fast-gate contracts catch observability marker drift deterministically without local-heavy lane leakage.
