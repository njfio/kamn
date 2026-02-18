# Spec - Issue #4127

- Title: Subtask: update observability governance docs and drift-contract tests for logging-telemetry closure
- Parent: #4119
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

Docs and checker expectations must remain synchronized for observability marker taxonomy and closure evidence.

## Objective

Validate docs-contract synchronization and drift detection for observability endpoint governance markers.

## Scope

In scope:
- Docs/marker parity assertions embedded in observability drift checker.
- Deterministic fail-closed drift behavior across source/docs marker surfaces.

Out of scope:
- Broad roadmap reprioritization.

## Acceptance Criteria

- AC-1: Observability docs/checker marker taxonomy remains synchronized.
- AC-2: Drift-contract suite fails closed on docs/source marker mismatch.
- AC-3: Baseline docs-contract path passes deterministically.

## Conformance Cases

- C-01 (AC-1/AC-2/AC-3): `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh` passes.

## Success Metrics

- Docs-contract divergence is caught deterministically before merge.
