# Issue #5221 Spec

- Title: Task: Add standalone-decision and typed-DID backlog conformance markers for data-layer M11/PRD
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
R43 states that Data Layer M11 hardening readiness and PRD conformance modules are intentionally standalone, but this rationale currently lives mostly in review narrative and is not enforced by deterministic docs contracts. Typed-DID migration backlog markers are also missing from planning docs.

## Scope
In:
- Add explicit standalone-decision markers and reason codes for M11 readiness and PRD conformance to `docs/planning/kamn-data-layer-prd.docx.md`.
- Add typed-DID migration backlog marker references to actionable follow-up issues.
- Add docs-contract tests that fail closed when markers drift.

Out:
- Implementing typed-DID migration changes.
- Refactoring Data Layer module runtime behavior.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 40
- shell_to_rust_ratio_delta_estimate: -0.0002
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: PRD planning doc includes explicit standalone-by-design markers for M11 readiness and PRD conformance, including reason-code taxonomy/version markers.
- AC-2: Typed-DID migration backlog markers reference actionable follow-up issue IDs.
- AC-3: Docs-contract tests assert required markers and fail closed when missing.
- AC-4: Targeted docs-contract tests pass with no shell LOC growth.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Load PRD planning doc text | Standalone-by-design markers and reason codes present |
| C-02 | AC-2 | Integration | Parse backlog issue marker list | At least one valid follow-up issue reference exists |
| C-03 | AC-3 | Regression | Remove marker and run contract test | Deterministic failing assertion |
| C-04 | AC-4 | Conformance | Run docs-contract suite | All marker assertions pass |

## Test Mapping
- C-01 -> `cargo test -p kamn-core --test data_layer_prd_standalone_decision_docs_contract standalone_decision_markers_present`
- C-02 -> `cargo test -p kamn-core --test data_layer_prd_standalone_decision_docs_contract typed_did_backlog_marker_references_follow_up_issue`
- C-03 -> RED evidence by temporarily validating stricter marker expectation in command scope
- C-04 -> `cargo test -p kamn-core --test data_layer_prd_standalone_decision_docs_contract`

## Success Metrics
- Deterministic marker contract exists and passes.
- Issue references in marker list include actionable follow-up (`#5223`).
- Shell LOC delta remains zero.
