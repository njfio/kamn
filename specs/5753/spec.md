# Spec: #5753 Reconcile R52 Post-Publication Priority Summary Status Markers

- Issue: #5753
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
R52 now includes additive post-publication reconciliation markers for quality gates and spec-volume
status, but Priority Summary rows remain historical baseline text and do not expose resolved
post-publication status in a deterministic marker contract. We need explicit reconciliation markers
that preserve baseline rows while making current status machine-verifiable.

## Scope
### In Scope
- Add post-publication priority-summary reconciliation marker contract guidance to
  `docs/review/README.md`.
- Add additive priority-summary reconciliation markers to
  `docs/review/gaps-and-issues-r52.md`.
- Extend docs-contract tests to enforce marker presence and consistency with existing
  post-publication quality-gate and spec-volume guardrail markers.
- Perform compensating single archived issue-spec pair cleanup
  (`specs/3872/ARCHIVED.md` + `specs/archive/3872/` + `specs/archive/index.md`) to preserve
  the `<= 693` non-regression cap after adding `specs/5753`.

### Out of Scope
- Rewriting historical R52 baseline rows in Section 8.
- Runtime/product behavior changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Priority reconciliation markers published
Given historical priority rows remain unchanged,
When post-publication priority reconciliation is recorded,
Then R52 includes additive markers capturing resolved status for critical CLI compile,
medium activity-ratio marker parsing, and high spec-volume guardrail posture.

### AC-2 Cross-section consistency
Given reconciliation markers are present,
When docs-contract tests parse values,
Then priority reconciliation values match existing post-publication quality-gate and
spec-volume guardrail reconciliation markers.

### AC-3 Snapshot preservation
Given R52 snapshot semantics,
When additive reconciliation markers are introduced,
Then original baseline priority table rows remain unchanged.

### AC-4 Fail-closed enforcement
Given marker contract keys are required,
When markers drift or are removed,
Then docs-contract tests fail.

### AC-5 Non-regression cap preservation
Given issue lifecycle artifacts add one `specs/<issue-id>` directory,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): `docs/review/gaps-and-issues-r52.md` contains priority reconciliation marker block.
- C-02 (AC-2/AC-4): `cargo test -p kamn-core --test review_r52_branch_hygiene_reconciliation_docs_contract`.
- C-03 (AC-3): baseline Priority Summary table rows in Section 8 remain unchanged.
- C-04 (AC-5): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-05 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-06 (AC-2/AC-4/AC-5): `cargo fmt --all --check`.
- C-07 (AC-2/AC-4/AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- Priority reconciliation markers are present and parsed by docs-contract tests.
- Marker values are consistent with existing post-publication sections.
- Spec-dir count remains at cap-compliant value (`<= 693`).
