# Spec: #5774 Reconcile R50 Doc-Contract Non-Regression Cap After R53 Lane Addition

- Issue: #5774
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
After merging #5773, the workspace-level gate fails in
`review_r50_doc_contract_consolidation_docs_contract` because current doc-contract test-file count is
`96` while the active R50 non-regression ratchet markers and assertions remain fixed at `95`.

## Scope
### In Scope
- Reconcile R50 non-regression marker values and matching test assertions with current suite count.
- Preserve fail-closed consolidation invariants and docs-contract behavior.
- Add lifecycle artifacts and milestone slice tracking.
- Perform compensating archived issue-spec pair cleanup so top-level `specs/` count remains `<= 693`.

### Out of Scope
- Broad redesign of docs-contract consolidation policy.
- CI/workflow changes.

## Acceptance Criteria
### AC-1 R50 non-regression marker contract reconciled
Given current docs-contract test-file count is `96`,
When R50 non-regression markers are updated,
Then marker values are internally consistent and match enforced expectations.

### AC-2 Failing lane restored to green
Given current failing lane in kamn-core docs-contract tests,
When reconciliation is implemented,
Then `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract` passes.

### AC-3 Related review-contract lanes remain green
Given R50/R53 marker interactions,
When reconciliation is applied,
Then related targeted lanes pass without regressions.

### AC-4 Workspace gate green
Given this regression was detected by workspace gate,
When fix is complete,
Then `cargo test --workspace --locked --all-features --no-fail-fast` passes.

### AC-5 Specs cap preserved
Given one new lifecycle directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): R50 markers for doc-contract non-regression baseline/max reflect reconciled deterministic count.
- C-02 (AC-2): `cargo test -p kamn-core --test review_r50_doc_contract_consolidation_docs_contract`.
- C-03 (AC-3): `cargo test -p kamn-core --test review_r53_docs_contract`.
- C-04 (AC-3): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-05 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-06 (AC-1..AC-5): `cargo fmt --all --check`.
- C-07 (AC-1..AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.
- C-08 (AC-4): `cargo test --workspace --locked --all-features --no-fail-fast`.

## Success Metrics / Observable Signals
- Failing R50 consolidation lane returns to green.
- Workspace full gate passes with no docs-contract non-regression failures.
- Top-level `specs/` count remains at or under cap.
