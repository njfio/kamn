# Spec: #5778 Reconcile R53 Portable-Agent Stalled Markers After Query-Surface Delivery

- Issue: #5778
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P1

## Problem Statement
`docs/review/gaps-and-issues-r53.md` is a snapshot report pinned to commit `982d52df`, but it currently lacks explicit post-publication reconciliation markers for portable-agent movement after PR `#5777` (issue `#5776`). Without this addendum, the report can be interpreted as still-currently-stalled even after new portable-agent query surfaces shipped and merged.

## Scope
### In Scope
- Add a post-publication reconciliation section and markers in `docs/review/gaps-and-issues-r53.md` for portable-agent status after `#5777`.
- Preserve snapshot semantics (baseline still tied to `982d52df`).
- Extend `crates/kamn-core/tests/review_r53_docs_contract.rs` with fail-closed assertions for new markers and consistency checks.
- Update R52 milestone tracking for this issue lifecycle.

### Out of Scope
- Recomputing all R53 baseline metrics/tables from scratch.
- New runtime protocol or API behavior.

## Acceptance Criteria
### AC-1 Post-publication portable-agent reconciliation markers exist
Given R53 remains a snapshot document,
When readers inspect post-publication sections,
Then they can see explicit markers proving portable-agent status moved from stagnant to advanced after `#5777`.

### AC-2 Snapshot semantics remain explicit
Given baseline metrics are tied to `982d52df`,
When reconciliation content is added,
Then baseline snapshot semantics remain unchanged and clearly labeled.

### AC-3 Docs-contract lane validates new markers
Given marker contracts are enforced in tests,
When `review_r53_docs_contract` runs,
Then it fail-closes for missing/inconsistent new portable-agent post-publication markers.

### AC-4 Spec cap remains within guardrail
Given top-level `specs/` cap is `<= 693`,
When `specs/5778` is added,
Then compensating archive cleanup preserves the cap.

## Conformance Cases
- C-01 (AC-1/AC-2/AC-3): `cargo test -p kamn-core --test review_r53_docs_contract`.
- C-02 (AC-4): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json /tmp/5778-spec-archive-check.json`.
- C-03 (AC-1..AC-4): `cargo fmt --all --check`.
- C-04 (AC-1..AC-4): `cargo clippy -p kamn-core --test review_r53_docs_contract -- -D warnings`.
- C-05 (AC-1..AC-4): `cargo test --workspace --locked --all-features --no-fail-fast`.

## Success Metrics / Observable Signals
- R53 review doc has explicit portable-agent post-publication reconciliation markers.
- Docs-contract lane enforces marker presence and consistency.
- Spec archive checker remains green with pointer/index parity.
