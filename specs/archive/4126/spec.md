# Spec - Issue #4126

- Title: Subtask: add ci smoke checker for observability marker lineage with local-heavy scrape exclusion policy
- Parent: #4119
- Milestone: R27.17 Structured logging and telemetry emission governance
- Status: Implemented
- Priority: P1

## Problem Statement

CI-fast gate must validate observability lineage contracts without running heavy scrape lanes by default.

## Objective

Validate deterministic observability marker-lineage checker behavior and CI exclusion boundaries for local-heavy scrape lanes.

## Scope

In scope:
- Observability endpoint drift checker lineage contracts.
- CI exclusion policies for local observability scrape and prometheus metrics run-mode commands.

Out of scope:
- Always-on heavy scrape execution in PR CI.

## Acceptance Criteria

- AC-1: Drift checker fails closed on marker lineage mismatch.
- AC-2: CI-fast gate excludes local-heavy observability scrape lanes deterministically.
- AC-3: CI-fast gate excludes local-heavy prometheus metrics lanes deterministically.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_observability_endpoint_drift_contract.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_local_observability_scrape_ci_exclusion_policy.sh` passes.
- C-03 (AC-3): `bash scripts/ci/test_service_api_prometheus_metrics_ci_exclusion_policy.sh` passes.

## Success Metrics

- Observability marker lineage and CI cost boundaries remain deterministic and fail closed.
