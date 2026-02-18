# Spec - Issue #3833

- Title: Subtask: move peer lifecycle and queue coordination logic to runtime_peer_phase module
- Parent: #3831
- Milestone: R27.2 Runtime architecture extraction (runtime.rs modularization)
- Status: Implemented
- Priority: P1

## Problem Statement

This R27.2 issue requires deterministic runtime modularization governance evidence with fail-closed contracts.

## Objective

Close this issue with AC-to-test traceability for runtime phase extraction, ownership docs contracts, and anti-regrowth budget policy.

## Scope

In scope:
- Deterministic contract validation for mapped runtime extraction suites.
- Fail-closed policy and docs/marker drift governance.
- Lifecycle artifact closure traceability.

Out of scope:
- Runtime behavior redesign.

## Acceptance Criteria

- AC-1: Runtime modularization/ownership/budget behavior remains deterministically validated.
- AC-2: Policy or docs-marker drift fails closed with stable signals.
- AC-3: Conformance evidence is deterministic and green.

## Conformance Cases

- C-01 (AC-1/AC-2): 'cargo test -p kamn-core --test runtime_module_extraction_contract' passes.
- C-02 (AC-1/AC-2): 'bash scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh' passes.
- C-03 (AC-3): all suites above pass in closure verification.
## Success Metrics

- R27.2 runtime extraction governance checks stay deterministic, auditable, and bounded.
