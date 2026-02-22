# Spec: #5759 Reconcile R52 Post-Publication Code-Quality Status Markers

- Issue: #5759
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 includes post-publication quality-gate reconciliation markers in Section 1.6, but Section 4.2
("Quality Gate Regression — BROKEN MAIN") remains baseline-only narrative without additive
post-publication status markers. We need explicit marker reconciliation so current status is
machine-verifiable while preserving historical snapshot wording.

## Scope
### In Scope
- Add post-publication code-quality status reconciliation marker contract guidance to
  `docs/review/README.md`.
- Add additive Section 4.3 marker block to `docs/review/gaps-and-issues-r52.md`.
- Extend docs-contract tests for marker presence and consistency with existing post-publication
  quality-gate markers.
- Perform compensating single archived issue-spec pair cleanup for issue `3874` to preserve
  top-level `specs/` non-regression cap (`<= 693`) after adding `specs/5759`.

### Out of Scope
- Rewriting historical Section 4.2 baseline text.
- Runtime behavior changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Code-quality reconciliation markers published
Given baseline Section 4.2 text remains unchanged,
When post-publication status reconciliation is recorded,
Then R52 includes additive markers describing post-publication code-quality status.

### AC-2 Marker consistency
Given additive code-quality markers and existing quality-gate reconciliation markers,
When docs-contract tests parse values,
Then code-quality markers are internally consistent and aligned with quality-gate markers.

### AC-3 Snapshot preservation
Given R52 snapshot semantics,
When additive Section 4.3 markers are added,
Then baseline Section 4.2 text remains unchanged.

### AC-4 Fail-closed enforcement
Given code-quality marker contract keys are required,
When markers drift or are removed,
Then docs-contract tests fail.

### AC-5 Non-regression cap preservation
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): `docs/review/gaps-and-issues-r52.md` contains Section 4.3 marker block.
- C-02 (AC-2/AC-4): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-03 (AC-3): Section 4.2 baseline lines remain unchanged.
- C-04 (AC-5): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-05 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-06 (AC-2/AC-4/AC-5): `cargo fmt --all --check`.
- C-07 (AC-2/AC-4/AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Section 4.3 markers are present and validated by docs-contract tests.
- Marker values align with existing quality-gate reconciliation status.
- Top-level `specs/` directory count remains cap-compliant (`<= 693`).
