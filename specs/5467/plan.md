# Issue #5467 Plan - Completed Milestone Closure Hygiene

## Approach
1. RED: add docs-contract test expecting a new closure-wave artifact and markers before artifact exists.
2. Capture pre-closure milestone state via GitHub API.
3. Close milestones `#94`, `#95`, `#96` (all with `open_issues=0`).
4. Publish planning artifact with pre/post evidence and deterministic markers.
5. GREEN: rerun docs-contract test and format checks.

## Affected Modules
- `docs/planning/2026-02-21-r49-completed-milestone-closure-wave.md` (new)
- `crates/kamn-core/tests/review_r49_completed_milestone_closure_docs_contract.rs` (new)
- `specs/milestones/r49-2-completed-milestone-closure-hygiene-wave/index.md`
- `specs/5467/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: milestone closure attempt fails due to transient counter state.
  - Mitigation: verify `open_issues=0` immediately before closure patch and capture evidence.
- Risk: closure evidence drift.
  - Mitigation: enforce marker presence/count in docs-contract test.

## Interfaces / Contracts
- `completed_milestone_closure_wave_schema_version=kamn.review.completed-milestone-closure-wave.v1`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test review_r49_completed_milestone_closure_docs_contract -- --nocapture`
- GREEN/REGRESSION:
  - `cargo test -p kamn-core --test review_r49_completed_milestone_closure_docs_contract -- --nocapture`
  - `cargo fmt --check`
