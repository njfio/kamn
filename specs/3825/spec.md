# Spec - Issue #3825

- Title: Subtask: add runbook marker definitions for signal intake, drain completion, and timeout fail-closed reasons
- Parent: #3824
- Milestone: R27.1 Runtime operations hardening (signals + lifecycle drain)
- Status: Implemented
- Priority: P1

## Problem Statement

This R27.1 issue requires deterministic signal handling and graceful shutdown governance evidence with fail-closed contracts.

## Objective

Close this issue with AC-to-test traceability for OS signal orchestration, lifecycle drain, runbook parity, and go/no-go evidence.

## Scope

In scope:
- Deterministic contract validation for mapped signal/shutdown suites.
- Fail-closed policy and marker/docs drift governance.
- Lifecycle artifact closure traceability.

Out of scope:
- Runtime behavior redesign outside shutdown orchestration governance.

## Acceptance Criteria

- AC-1: Signal/shutdown/governance behavior remains deterministically validated.
- AC-2: Policy or runbook-marker drift fails closed with stable signals.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1/AC-2): 'bash scripts/runtime/test_check_service_api_graceful_shutdown_drain_live_policy.sh' passes.
- C-02 (AC-1/AC-2): 'bash scripts/runtime/test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh' passes.
- C-03 (AC-3): all suites above pass in closure verification.
## Success Metrics

- R27.1 signal and graceful-shutdown governance checks stay deterministic, auditable, and bounded.
