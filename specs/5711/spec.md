# Spec: #5711 Reconcile R52 Post-Publication Quality-Gate Status Markers

- Issue: #5711
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P2

## Problem Statement
`docs/review/gaps-and-issues-r52.md` captures an as-of snapshot that reports broken-main quality gates
(kamn-cli compile break and marker-parser failures). Those regressions were fixed post-publication, but
without deterministic reconciliation markers the current green-main status is not machine-verifiable.

## Scope
### In Scope
- Add a post-publication quality-gate reconciliation marker schema to `docs/review/README.md`.
- Add an R52 post-publication quality-gate reconciliation subsection in `docs/review/gaps-and-issues-r52.md`.
- Extend existing R52 docs-contract test coverage (in an existing file) to fail closed on marker drift.

### Out of Scope
- Rewriting R52 as-of snapshot baseline values.
- CI/workflow structural changes.
- Feature/runtime behavior changes.

## Acceptance Criteria
### AC-1 Deterministic marker schema and values
Given `docs/review/gaps-and-issues-r52.md`,
When the post-publication quality-gate reconciliation section is parsed,
Then required markers exist and indicate resolved status for the previously reported R52 quality-gate regressions.

### AC-2 Fail-closed contract enforcement
Given reconciliation marker drift/removal in the R52 review artifact or README marker guidance,
When docs-contract tests run,
Then targeted docs-contract tests fail deterministically.

### AC-3 Snapshot semantics preserved
Given R52 review snapshot semantics,
When post-publication reconciliation markers are added,
Then snapshot baseline claims remain explicitly historical and are not overwritten.

### AC-4 Verification gates pass
Given this slice implementation,
When verification commands run,
Then `cargo fmt --all --check`, `cargo clippy -p kamn-core --tests -- -D warnings`, and targeted docs-contract tests pass.

## Conformance Cases
- C-01 (AC-1, AC-2): targeted R52 docs-contract tests enforce README schema and R52 markers.
- C-02 (AC-3): R52 snapshot header and as-of baseline statements remain present after reconciliation addition.
- C-03 (AC-4): `cargo fmt --all --check`.
- C-04 (AC-4): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- R52 artifact exposes deterministic post-publication quality-gate reconciliation markers.
- Existing docs-contract lane fails closed if markers drift.
- No change to original as-of snapshot baseline statements.
