# Spec: Issue #6088 - Extract kamn-core Phase-1 Live Probe Matrix Slice

- Issue: #6088
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r67-runtime-hardening-and-surface-reduction/index.md`
- Last Updated: 2026-02-26
- Parent: #6085

## Problem Statement
`kamn-core` remains oversized and mixes runtime slices with weak crate boundaries. The `live_probe_matrix` surface is a cohesive, low-coupling runtime contract that can be extracted in phase 1 without behavior drift, reducing `kamn-core` ownership breadth while preserving existing API imports.

## Scope
In scope:
- Extract `live_probe_matrix` implementation from `kamn-core` into a focused crate.
- Preserve `kamn_core::{LiveProbeMatrix*}` compatibility via re-export shim.
- Preserve behavior parity via existing contract tests and new crate-local tests.
- Wire workspace membership and dependencies for the extracted crate.

Out of scope:
- Additional `kamn-core` slice extractions beyond `live_probe_matrix`.
- Public API redesign for probe-matrix types.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: `live_probe_matrix` implementation is moved to a dedicated crate `crates/kamn-live-probe-matrix` with equivalent public types/functions.
- AC-2: `kamn-core` preserves backwards-compatible exports (`kamn_core::LiveProbeMatrix*`) via compatibility re-exports.
- AC-3: Existing `kamn-core` probe-matrix contract tests pass without caller-side API changes.
- AC-4: New crate-local unit tests cover entry validation, duplicate row rejection, and fail-closed aggregate semantics.

## Conformance Cases
- C-01 (Conformance, AC-1): Workspace includes `crates/kamn-live-probe-matrix`, and crate exposes `LiveProbeMatrixMode`, `LiveProbeMatrixStatus`, `LiveProbeMatrixEntry`, `LiveProbeMatrixReport`, `LiveProbeMatrixError`.
- C-02 (Conformance, AC-2): `crates/kamn-core/src/live_probe_matrix.rs` is a compatibility re-export module targeting `kamn_live_probe_matrix`.
- C-03 (Regression, AC-3): `cargo test -p kamn-core --test live_probe_matrix_contract` passes unchanged.
- C-04 (Unit/Functional, AC-4): `cargo test -p kamn-live-probe-matrix` passes, including fail-closed aggregate and duplicate detection tests.

## Success Metrics / Observable Signals
- `kamn-core` line ownership is reduced for one runtime slice without API break.
- No regressions in probe-matrix behavior observable through existing `kamn-core` contract tests.
- New extracted crate has explicit unit coverage for core invariants.
