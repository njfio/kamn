# Spec - Issue #3813

- Title: Epic: R27.1 harden runtime operations with real signal handling and lifecycle drain guarantees
- Parent: #3812
- Milestone: R27.1 Runtime operations hardening (signals + lifecycle drain)
- Status: Implemented
- Priority: P0

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

- C-01 (AC-1/AC-2): 'bash scripts/ci/test_run_daemon_os_signal_reproducer.sh' passes.
- C-02 (AC-1/AC-2): 'bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh' passes.
- C-03 (AC-1/AC-2): 'bash scripts/runtime/test_validate_daemon_os_signal_live.sh' passes.
- C-04 (AC-1/AC-2): 'bash scripts/runtime/test_check_local_signal_secret_hygiene_live_policy.sh' passes.
- C-05 (AC-1/AC-2): 'bash scripts/runtime/test_validate_local_signal_secret_hygiene_live_contract_lane.sh' passes.
- C-06 (AC-1/AC-2): 'bash scripts/ci/test_local_signal_secret_hygiene_ci_exclusion_policy.sh' passes.
- C-07 (AC-1/AC-2): 'bash scripts/runtime/test_check_service_api_graceful_shutdown_drain_live_policy.sh' passes.
- C-08 (AC-1/AC-2): 'bash scripts/runtime/test_validate_service_api_graceful_shutdown_drain_live_contract_lane.sh' passes.
- C-09 (AC-1/AC-2): 'bash scripts/ci/test_service_api_graceful_shutdown_drain_ci_exclusion_policy.sh' passes.
- C-10 (AC-1/AC-2): 'bash scripts/runtime/test_check_service_api_shutdown_abrupt_close_regression_live_policy.sh' passes.
- C-11 (AC-1/AC-2): 'bash scripts/runtime/test_validate_service_api_shutdown_abrupt_close_regression_live_contract_lane.sh' passes.
- C-12 (AC-1/AC-2): 'bash scripts/ci/test_service_api_shutdown_abrupt_close_regression_ci_exclusion_policy.sh' passes.
- C-13 (AC-1/AC-2): 'bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh' passes.
- C-14 (AC-1/AC-2): 'bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh' passes.
- C-15 (AC-1/AC-2): 'bash scripts/runtime/test_run_lifecycle_property_contract_lane.sh' passes.
- C-16 (AC-3): all suites above pass in closure verification.
## Success Metrics

- R27.1 signal and graceful-shutdown governance checks stay deterministic, auditable, and bounded.
