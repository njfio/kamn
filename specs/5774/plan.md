# Plan: #5774 Reconcile R50 Doc-Contract Non-Regression Cap After R53 Lane Addition

## Approach
1. Add lifecycle artifacts and milestone slice tracking for #5774.
2. RED: rerun failing R50 consolidation lane and record failure evidence.
3. Implement marker and test expectation reconciliation for R50 non-regression count (`95 -> 96`).
4. Compensating cleanup: remove one archived issue-spec pair and update `specs/archive/index.md`.
5. GREEN: rerun targeted docs-contract lanes + archive policy checker.
6. Verify fmt/clippy and rerun workspace gate.
7. Closure: set spec status `Implemented`, mark tasks complete, update milestone index, close issue.

## Affected Modules / Files
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_doc_contract_consolidation_docs_contract.rs`
- `specs/archive/index.md`
- `specs/5774/spec.md`
- `specs/5774/plan.md`
- `specs/5774/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental weakening of non-regression semantics.
  - Mitigation: keep baseline/max equality invariant intact and adjust only deterministic count value.
- Risk: spec-cap regression due new lifecycle directory.
  - Mitigation: compensating archive cleanup + policy checker validation.
- Risk: targeted-lane green but workspace still fails.
  - Mitigation: rerun full workspace gate before closure.

## Interfaces / Contracts
Primary contract remains `review_r50_doc_contract_consolidation_docs_contract` plus related review lanes.

## ADR
No ADR required (docs/test/governance-only change).
