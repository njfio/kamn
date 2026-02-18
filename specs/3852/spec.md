# Spec - Issue #3852

- Title: Task: implement unified governance budget report across script, harness, and ignored-test surfaces
- Parent: #3851
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Governance budget telemetry was distributed across multiple script surfaces, increasing blind spots and operational overhead.

## Objective

Deliver deterministic unified budget governance reporting and fail-closed policy validation across script-surface trends, harness-budget checks, and ignored-test inventory governance.

## Scope

In scope:
- Combined shell-surface trend report generation and policy checks.
- Ignored-test inventory and script-trend composed contract-lane verification.
- Generic test-harness soft-budget contract-lane verification as part of unified governance evidence.

Out of scope:
- Live topology orchestration and unrelated runtime feature work.

## Acceptance Criteria

- AC-1: Unified governance budget report generation is deterministic and schema-validated.
- AC-2: Threshold policy enforcement is deterministic with explicit fail-code taxonomy.
- AC-3: Ignored-test + script-trend composed governance lane is executable and fail-closed.
- AC-4: Harness budget governance remains covered in the unified validation suite.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_generate_combined_shell_surface_trend_report.sh` passes.
- C-02 (AC-2): `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` passes.
- C-03 (AC-3): `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh` passes.
- C-04 (AC-4): `bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh` passes.

## Success Metrics

- Budget governance evidence remains deterministic and executable across script, harness, and ignored-test surfaces.
- Task closure is auditable with direct AC-to-test traceability.
