# Spec: #5750 Reconcile R52 Post-Publication Spec-Volume Guardrail Status Markers

- Issue: #5750
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
R52 baseline sections intentionally preserve an as-of snapshot that reports spec-volume guardrail
status as severely breached (`845`, `9.2:1`). Post-publication remediation reduced top-level
`specs/` directories to `693`, which is within the `7.7` ratio guardrail. We need additive
reconciliation markers so current status is machine-verifiable without rewriting historical snapshot
content.

## Scope
### In Scope
- Add post-publication spec-volume guardrail reconciliation section + markers to
  `docs/review/gaps-and-issues-r52.md`.
- Extend docs-contract tests to enforce marker presence and consistency.
- Preserve baseline snapshot sections unchanged.
- Perform a compensating single archived issue-spec pair cleanup (pointer + payload + archive index row)
  so the repository remains within the R50 non-regression spec-dir cap after adding `specs/5750`.

### Out of Scope
- Additional archive deletion tranches.
- Runtime behavior changes.
- CI/workflow topology changes.

## Acceptance Criteria
### AC-1 Reconciliation markers published
Given historical snapshot sections remain unchanged,
When post-publication reconciliation is recorded,
Then R52 includes additive markers for current spec-dir count, module count, ratio, target max,
and resolved status.

### AC-2 Marker consistency and guardrail pass condition
Given reconciliation markers are present,
When docs-contract tests parse marker values,
Then marker math is internally consistent and `current_ratio <= target_ratio_max`.

### AC-3 Snapshot preservation
Given the R52 report baseline snapshot contract,
When reconciliation content is added,
Then original snapshot counts/status lines remain intact and are not rewritten.

### AC-4 Fail-closed docs-contract enforcement
Given reconciliation markers are contract data,
When marker values drift or are removed,
Then docs-contract tests fail.

### AC-5 Non-regression cap preservation
Given issue lifecycle artifacts add one new `specs/<issue-id>` directory,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): `docs/review/gaps-and-issues-r52.md` contains reconciliation marker block.
- C-02 (AC-2): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-03 (AC-3): baseline snapshot markers in `docs/review/gaps-and-issues-r52.md` remain unchanged.
- C-04 (AC-4): RED/GREEN evidence from targeted docs-contract test.
- C-05 (AC-5): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-06 (AC-2/AC-4/AC-5): `cargo fmt --all --check`.
- C-07 (AC-2/AC-4/AC-5): `cargo clippy -p kamn-core --tests -- -D warnings`.

## Success Metrics / Observable Signals
- R52 report includes explicit post-publication guardrail reconciliation markers.
- Targeted docs-contract test validates marker math and guardrail pass condition.
- Baseline snapshot section remains unchanged.
