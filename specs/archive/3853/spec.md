# Spec - Issue #3853

- Title: Subtask: add combined governance budget report generator and schema contract
- Parent: #3852
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Budget signals were fragmented and lacked a deterministic single schema for combined shell-surface trend telemetry.

## Objective

Provide deterministic combined-governance budget report generation with explicit schema markers and fixture-backed contract coverage.

## Scope

In scope:
- Combined shell-surface trend report generation.
- Schema markers and deterministic output fields.
- Regression guardrails for missing baseline/config artifacts.

Out of scope:
- Runtime extraction and unrelated feature work.

## Acceptance Criteria

- AC-1: Combined trend report generator emits deterministic report markers and JSON schema.
- AC-2: Report output includes current/baseline/delta telemetry and script-budget checker summary.
- AC-3: Missing required baseline/config inputs fail closed with deterministic error markers.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh` passes.
- C-02 (AC-1/AC-2): `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh` passes.
- C-03 (AC-3): fail-closed missing-baseline scenario in `scripts/ci/test_generate_combined_shell_surface_trend_report.sh` passes.

## Success Metrics

- Combined generator output is deterministic and schema-validated.
- Regression/fail-closed behavior remains covered by executable contract tests.
