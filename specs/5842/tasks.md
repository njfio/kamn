# Tasks: Issue #5842

## Ordered Tasks
- [x] T1 (RED): add/extend failing docs-contract assertions for R56 unresolved markers, freeze enforcement, and corrected attribution semantics.
- [x] T2 (GREEN): implement deterministic freeze contract for R51+ review docs in existing review docs-contract lane.
- [x] T3 (GREEN): update tracked spec-dir counter(s) to explicit git-tree semantics and keep contamination regression passing.
- [x] T4 (GREEN): ensure production `expect()` inventory markers/logic are deterministic and consistent with scoped computation.
- [x] T5 (GREEN): reduce shell script LOC on high-noise CI surface while preserving command-surface behavior.
- [x] T6 (VERIFY): run targeted Rust + shell contract tests and confirm all AC mappings are green.
- [x] T7 (DOCS): update review marker documentation/policies for any new enforced marker schemas.

## Tier Mapping
- Unit: marker parsing/status derivation helpers, tracked-count helper behavior.
- Functional: R56 docs-contract marker checks and policy-enforcement assertions.
- Regression: untracked spec-dir contamination and post-publication review-edit freeze checks.
- Integration: CI tool/script contract lane checks for touched shell surfaces.
