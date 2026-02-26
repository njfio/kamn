# Spec: Issue #6033 - Add core invariants unit tests for data_layer_m10_partition_archival

- Issue: #6033
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5976

## Problem Statement
`data_layer_m10_partition_archival` has sparse helper-level coverage but no direct tests for partition lifecycle registry behavior (registration, archival transitions, reattach transitions, and recovery readiness projections).

## Scope
In scope:
- Add direct unit coverage for `data_layer_m10_partition_archival/registry.rs` contracts.
- Validate deterministic partition naming/ordering and archival-index projection ordering.
- Validate fail-closed transition paths and owner-neutral recovery-readiness decisions.

Out of scope:
- M8 integration end-to-end wiring.
- Phase-6 scheduler/runtime orchestration algorithm changes.
- Any production behavior changes.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Partition registration and future partition planning are deterministic and validated.
- AC-2: Archival evaluation archives only due shredded partitions and emits deterministic index entries.
- AC-3: Invalid lifecycle transitions fail closed while valid archived->reattached transitions and recovery readiness projections remain deterministic.

## Conformance Cases
- C-01 (Unit, AC-1): Register partitions and verify deterministic names plus future partition plan ordering.
- C-02 (Functional, AC-2): `archive_due_partitions` archives only due shredded active partitions and returns entries sorted by month/name with stable metadata markers.
- C-03 (Regression, AC-3): Reattaching an active partition fails with `InvalidLifecycleTransition` + stable reason code.
- C-04 (Conformance, AC-3): Archived partition with complete metadata reports `Ready`; active partition reports blocked/ineligible reason.

## Success Metrics / Observable Signals
- New registry-level M10 tests pass in `kamn-core`.
- AC-to-test mapping is explicit in PR verification table.
- M10 lifecycle contracts no longer rely solely on helper-only tests.
