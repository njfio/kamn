# Spec - Issue #3854

- Title: Subtask: enforce threshold policy with deterministic fail codes across budget surfaces
- Parent: #3852
- Milestone: R27.3 Live libp2p+Kolme proof and governance budgets
- Status: Implemented
- Priority: P1

## Problem Statement

Budget regressions require deterministic threshold-policy enforcement and explicit fail-code taxonomy so drift fails closed.

## Objective

Enforce combined shell-surface threshold policy with deterministic reason markers and fixture-backed failure-mode coverage.

## Scope

In scope:
- Combined shell-surface policy checker threshold enforcement.
- Deterministic reason taxonomy and reason-code markers.
- Regression scenarios for tampered reports and invalid threshold fixtures.

Out of scope:
- Release evidence bundling.

## Acceptance Criteria

- AC-1: Policy checker emits deterministic `status/policy_decision/trend_status` outputs.
- AC-2: Threshold breaches fail closed with deterministic reason codes.
- AC-3: Invalid threshold metadata/orders/values and stale threshold fixtures fail closed deterministically.

## Conformance Cases

- C-01 (AC-1): passing scenario in `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` returns `status=ok`, `policy_decision=GO`.
- C-02 (AC-2): tampered/fail-threshold scenarios in `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` return deterministic fail reason codes.
- C-03 (AC-3): invalid-order/invalid-value/stale-threshold scenarios in `bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh` fail closed with deterministic markers.
- C-04 (AC-1/AC-2): composed contract validation in `bash scripts/ci/test_run_ignored_test_and_script_budget_trend_contract_lane.sh` passes.

## Success Metrics

- Threshold governance remains deterministic and fail-closed across scripted validation surfaces.
- Policy drift is caught by executable regression checks with stable reason markers.
