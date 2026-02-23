# Spec: Issue #5810 - Reconcile Spec-Volume Cap Regression After #5808

- Issue: #5810
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-23

## Problem Statement
After merging `#5809`, top-level tracked `specs/<issue-id>/` directory count increased by one. The review-doc contract lane `review_r53_docs_contract` now fails the non-regression cap assertion (`top_level_spec_dir_count() <= non_regression_spec_dir_max`).

## Scope
In scope:
- Reproduce the failing docs-contract lane (RED evidence).
- Execute minimal bounded spec-cap preservation by removing the required number of legacy archived top-level spec pointer directories.
- Re-run docs-contract and harness regression lanes.
- Finalize lifecycle/milestone metadata.

Out of scope:
- Runtime or harness behavior changes.
- Review policy schema changes.
- Shell/workflow/template updates.

## Acceptance Criteria
- AC-1: `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` is red before remediation and green after remediation.
- AC-2: Top-level tracked spec directory count returns within non-regression cap.
- AC-3: `#5810` lifecycle artifacts are present and marked completed.
- AC-4: `#5808` harness regression lane remains green after cap remediation.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | pre-fix `review_r53_docs_contract` run | Fails at spec-dir cap assertion. |
| C-02 | AC-1/AC-2 | Conformance | post-fix `review_r53_docs_contract` run | Passes with cap assertion restored. |
| C-03 | AC-3 | Conformance | `specs/5810/{spec,plan,tasks}.md` + milestone index | Lifecycle markers present and finalized. |
| C-04 | AC-4 | Regression | `cargo test -p kamn-e2e-harness -- --nocapture` | Harness slice remains green. |

## Test Mapping
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture` (RED + GREEN)
- `cargo test -p kamn-e2e-harness -- --nocapture`

## Success Metrics / Observable Signals
- Docs-contract cap lane passes.
- No harness behavior regressions.
- Milestone index and issue lifecycle markers are closure-consistent.
