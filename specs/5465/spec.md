# Issue #5465 Spec - R49 Ignored-Test Periodic Re-Evaluation

- Status: Implemented
- Issue: #5465
- Parent: #5449
- Milestone: R49.1 Ignored-test periodic re-evaluation

## Problem Statement
R48 review explicitly marks ignored-test re-evaluation as due in R49. Without a scheduled audit refresh, the ignored-test inventory can become stale governance debt rather than intentional deep-lane opt-in policy.

## Scope
In scope:
- Recompute current ignored-test inventory and confirm alignment with baseline fixtures.
- Publish R49 disposition outcomes for every ignored test with explicit retain/promote/deprecate rationale.
- Add deterministic marker coverage in docs-contract tests.

Out of scope:
- Converting deep-lane ignored tests to always-on fast-lane execution.
- Rewriting ignored-test inventory checker implementation.

## Acceptance Criteria
- AC-1: A review/planning artifact records the current ignored-test inventory with deterministic evidence commands and count markers.
- AC-2: Every currently ignored test has explicit R49 disposition and rationale in the artifact.
- AC-3: Rust docs-contract tests enforce marker presence + inventory/disposition consistency.

## Conformance Cases
- C-01 (Functional, AC-1): Re-evaluation artifact contains inventory evidence commands and `ignored_test_inventory_count=12` marker.
- C-02 (Functional, AC-2): Artifact includes explicit disposition rows for all 12 baseline ignored tests.
- C-03 (Conformance, AC-3): `cargo test -p kamn-core --test review_r49_ignored_test_disposition_docs_contract -- --nocapture` passes.

## Success Metrics / Observable Signals
- R49 periodic review marker is published and machine-checkable.
- Ignored-test inventory remains baseline-aligned (12) with explicit rationale.
- Missing or drifted markers fail docs-contract tests.
