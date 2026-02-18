# Spec - Issue #3857

- Title: Subtask: add milestone closure summary generator and docs-contract synchronization checks
- Parent: #3855
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Milestone closure governance needs deterministic and auditable summary generation plus docs-contract synchronization to avoid drift.

## Objective

Automate closure-summary generation and enforce docs-plan synchronization through deterministic contract checks.

## Scope

In scope:
- Budget artifact summary generation for closure review.
- CI strategy docs-contract synchronization checks for required governance markers.

Out of scope:
- Network protocol/runtime feature changes.

## Acceptance Criteria

- AC-1: Closure summary generator produces deterministic aggregated telemetry output.
- AC-2: Lane-filter and all-lanes summary modes are validated.
- AC-3: Required CI strategy/governance markers remain synchronized via docs-contract checks.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/ci/test_summarize_budget_artifacts.sh` passes.
- C-02 (AC-3): `bash scripts/ci/test_ci_strategy_contract.sh` passes.

## Success Metrics

- Closure summaries remain reproducible.
- Docs-contract drift for closure governance markers is caught deterministically.
