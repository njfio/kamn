# Issue 6249 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
`kamn-core` still contains wave-1 compatibility shim modules that only re-export symbols from extracted crates (`kamn-runtime-guards`, `kamn-live-probe-matrix`, `kamn-bridges`). This keeps extraction incomplete, increases maintenance overhead, and obscures ownership boundaries.

## Scope
In scope:
- Inventory shim/facade modules in `kamn-core` with explicit keep/remove decisions.
- Retire removable shim modules by replacing module-level wrappers with direct extracted-crate re-exports in `kamn-core`.
- Migrate in-repo consumers (workspace crates) to use extracted crates directly for migrated surfaces.
- Document compatibility surfaces that remain and define an explicit removal timeline.

Out of scope:
- External downstream repository migration outside this workspace.
- Functional redesign of runtime-guard algorithms.

## Acceptance Criteria
- AC-1: Shim inventory is documented with keep/remove decision and rationale per shim.
- AC-2: Workspace consumers for migrated surfaces use extracted crates directly (not `kamn-core` shim modules).
- AC-3: Obsolete shim modules are retired from the primary export path in `kamn-core`; any temporary compatibility shims are explicitly hard-deprecated.
- AC-4: Compatibility timeline for any kept re-export surface is explicitly documented.
- AC-5: Unit, Functional, Integration, and Regression tests for migrated surfaces pass.

## Conformance Cases
- C-01 (AC-1, Functional): `docs/planning/r59-followup.md` includes shim inventory table with module path + decision.
- C-02 (AC-2, Integration): `kamn-node` anti-spam service-api path imports `kamn_runtime_guards::anti_spam::*` directly.
- C-03 (AC-3, Conformance): `kamn-core/src/lib.rs` no longer depends on shim modules for migrated exports, and remaining shim modules carry explicit deprecation markers.
- C-04 (AC-4, Functional): ADR/follow-up docs include explicit removal target milestone/date for temporary compatibility exports.
- C-05 (AC-5, Regression): Targeted tests for runtime guards and migrated consumer paths pass.

## Test Mapping
- Unit: guard contract tests in `kamn-runtime-guards` and `kamn-node` anti-spam helpers.
- Functional: shim inventory/doc contract checks.
- Integration: `kamn-node` service API anti-spam behavior remains unchanged.
- Regression: extraction-bridge and compatibility tests in `kamn-core` updated to assert direct extracted-crate linkage.
- Performance: N/A (module ownership and import-path migration only).
