# Spec - Issue #3888

- Title: Subtask: add convergence artifact schema and reason-taxonomy policy checker
- Parent: #3864
- Milestone: R26.1 Live libp2p network I/O and convergence
- Status: Implemented
- Priority: P1

## Problem Statement

This R26.1 issue requires deterministic libp2p convergence/recovery governance evidence with fail-closed policy checks.

## Objective

Close this issue with explicit AC-to-test traceability for the live libp2p convergence and recovery contract surfaces.

## Scope

In scope:
- Deterministic contract validation for the mapped R26.1 libp2p suites.
- Fail-closed policy and marker drift governance.
- Lifecycle artifact closure traceability.

Out of scope:
- Transport protocol redesign.

## Acceptance Criteria

- AC-1: Deterministic libp2p convergence/recovery/governance behavior remains validated.
- AC-2: Drift in policy/marker contracts fails closed with stable signals.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1/AC-2): bash scripts/runtime/test_check_libp2p_convergence_process_isolated_live_policy.sh passes.
- C-02 (AC-1/AC-2): bash scripts/runtime/test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh passes.
- C-03 (AC-3): all suites above pass in closure verification.
## Success Metrics

- R26.1 libp2p convergence and recovery governance checks stay deterministic and auditable.
