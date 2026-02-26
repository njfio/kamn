# Spec: Issue #6035 - Add core invariants unit tests for data_layer_m8_compliance_lifecycle

- Issue: #6035
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs` currently has helper-level tests only and lacks direct registry-level coverage for owner-scoped retention, legal-hold, and crypto-shred lifecycle contracts.

## Scope
In scope:
- Add direct unit tests for `DataLayerM8ComplianceRegistry` behavior.
- Validate deterministic sequence assignment and retention-due projection ordering.
- Validate fail-closed legal-hold behavior for crypto-shred transitions.
- Validate owner-scope boundary enforcement and exclusion of held/shredded records from due projections.

Out of scope:
- Cross-service integration wiring.
- Changes to M8 production API behavior.
- M9+ module coverage.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Registry assigns deterministic sequence numbers and projects retention-due candidates deterministically.
- AC-2: Crypto-shred is fail-closed while legal hold is active and tombstones wrapped keys after authorized shred.
- AC-3: Owner-scope enforcement remains strict and retention-due projection excludes legal-held or already-shredded records.

## Conformance Cases
- C-01 (Unit, AC-1): Register two records and verify deterministic sequence/order in retention due projection.
- C-02 (Regression, AC-2): Legal-hold-active message rejects crypto-shred; after release, shred applies tombstone marker and repeat shred fails closed.
- C-03 (Conformance, AC-3): Owner-scope mismatch fails with stable reason code; due projection includes only eligible records.

## Success Metrics / Observable Signals
- New registry-level M8 tests pass in `kamn-core`.
- AC-to-test mapping is explicit in PR verification table.
- M8 lifecycle registry no longer has only helper-level direct coverage.
