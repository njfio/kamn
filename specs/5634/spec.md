# Spec: #5634 EVIDENCE Step Inventory Parity

- Issue: #5634
- Milestone: R55 E2E Evidence Step Inventory Parity
- Status: Reviewed
- Priority: P1

## Problem Statement
`EVIDENCE` phase currently uses a single synthetic step marker. PRD section 11.2 defines a six-step evidence lifecycle. The harness run contract should expose deterministic per-step evidence markers while preserving existing phase-level pass/fail semantics.

## Scope
### In Scope
- Replace single EVIDENCE step marker with PRD-aligned deterministic step inventory.
- Keep phase status derived from step aggregation and evidence contract semantics.
- Preserve output schema and runtime/live marker contracts.

### Out of Scope
- Real chain dump or snapshot I/O execution.
- External process orchestration changes.

## Acceptance Criteria
### AC-1 Normal-path evidence inventory
Given non-failing evidence path,
When run output is generated,
Then EVIDENCE phase renders PRD-aligned six-step inventory and all evidence steps are `PASS`.

### AC-2 Evidence-fail inventory propagation
Given deterministic evidence failure path (`evidence-fail`),
When run output is generated,
Then evidence contract-sensitive steps render `FAIL` and EVIDENCE phase status is `FAIL`.

### AC-3 Lifecycle summary propagation
Given evidence step-inventory activation,
When lifecycle summary is computed,
Then step totals reflect expanded evidence inventory for normal and fail paths.

### AC-4 Contract stability
Given existing runtime/live execution contracts,
When evidence-step activation is applied,
Then existing non-evidence marker contracts remain stable.

## Conformance Cases
- C-01 (AC-1): sdk-direct run includes six evidence steps with `PASS` statuses.
- C-02 (AC-2): evidence-fail run includes fail statuses on evidence verification/finalization steps and phase `FAIL`.
- C-03 (AC-3): lifecycle step totals update from expanded evidence step inventory.
- C-04 (AC-4): existing runtime/live contract tests remain green.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new evidence inventory assertions.
- `cargo test -p kamn-e2e-harness` remains green.
