# Spec: #5690 Consolidate Harness Doc-Contract Test Files

- Issue: #5690
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P2

## Problem Statement
`kamn-e2e-harness` currently maintains 41 separate `*_docs_contract.rs` files.
This file-level fragmentation adds maintenance overhead and contributes to
governance-heavy churn.

## Scope
### In Scope
- Consolidate existing docs-contract tests into fewer grouped files.
- Preserve all marker assertions and milestone reference assertions.
- Keep harness test suite behavior unchanged.

### Out of Scope
- Removing required contract assertions.
- Editing source docs marker content.

## Acceptance Criteria
### AC-1 File-count reduction
Given baseline docs-contract file inventory,
When consolidation is complete,
Then the count of `*_docs_contract.rs` files in harness tests is materially lower.

### AC-2 Assertion parity
Given existing docs-contract coverage,
When grouped tests run,
Then all prior assertions remain represented and passing.

### AC-3 Regression stability
Given full harness tests,
When refactor is merged,
Then `cargo test -p kamn-e2e-harness` remains green.

## Conformance Cases
- C-01 (AC-1): baseline vs post-change docs-contract file count telemetry.
- C-02 (AC-2): consolidated docs-contract tests pass.
- C-03 (AC-3): full crate test suite passes after consolidation.

## Success Metrics
- Harness docs-contract test file count reduced from baseline.
- `cargo test -p kamn-e2e-harness` passes with no assertion loss.
