# Issue #5219 Spec

- Title: Task: Add governance-to-feature activity ratio markers to release review flow
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
R43 reported governance-vs-feature activity as narrative text (~95% governance / ~5% features), but there is no deterministic marker schema or contract test in the release review flow. Drift or omission would go undetected.

## Scope
In:
- Define marker schema for governance-vs-feature commit activity in `docs/review/gaps-and-issues-r*.md` artifacts.
- Add docs-contract tests that enforce marker presence and numeric parseability for R43+ review artifacts.
- Document schema usage in `docs/review/README.md`.

Out:
- Enforcing hard fail thresholds on the ratio itself.
- Retroactively backfilling pre-R43 review reports.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 80
- shell_to_rust_ratio_delta_estimate: -0.0005
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Release review artifacts (R43+) include governance-to-feature activity ratio markers.
- AC-2: Marker schema is documented and parse-stable.
- AC-3: CI-facing docs-contract tests fail when marker format/fields drift.
- AC-4: Targeted tests/clippy pass with no shell LOC increase.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Parse R43 gaps-and-issues review doc | Required marker keys present |
| C-02 | AC-2 | Integration | Parse marker values from review docs + README schema reference | Numeric fields parse and schema key is documented |
| C-03 | AC-3 | Regression | Run marker contract test with missing/invalid marker setup | Deterministic failure when required marker absent/invalid |
| C-04 | AC-4 | Conformance | Run targeted tests + clippy | All checks pass; shell delta stays zero |

## Test Mapping
- C-01 -> `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract functional_release_review_activity_ratio_markers_present_for_r43_and_later`
- C-02 -> `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract integration_release_review_activity_ratio_markers_are_numeric_and_consistent`
- C-03 -> RED run before marker block exists (contract test fails)
- C-04 -> `cargo test -p kamn-core --test release_review_activity_ratio_docs_contract` + `cargo clippy -p kamn-core --test release_review_activity_ratio_docs_contract -- -D warnings`

## Success Metrics
- R43+ release review docs have deterministic marker block.
- Docs-contract suite enforces schema and numeric consistency.
- shell_loc_delta_actual = 0.
