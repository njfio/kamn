# Plan: #5772 Complete R53 Review Marker Contract Coverage

## Approach
1. Add lifecycle artifacts and mark milestone slice 34 in progress.
2. RED: add new R53 docs-contract tests asserting required marker presence/invariants; run targeted lane expecting failure.
3. Implement: append missing marker blocks to `docs/review/gaps-and-issues-r53.md` with deterministic values.
4. Implement: compensating archive cleanup for one archived issue-spec pair + update `specs/archive/index.md`.
5. GREEN: rerun R53 docs-contract + spec-volume/archive policy checks.
6. Verify formatting/lint and finalize closure artifacts.

## Affected Modules / Files
- `docs/review/gaps-and-issues-r53.md`
- `crates/kamn-core/tests/review_r53_docs_contract.rs` (new)
- `specs/archive/index.md`
- `specs/5772/spec.md`
- `specs/5772/plan.md`
- `specs/5772/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: accidental marker inconsistency with existing R53 narrative numbers.
  - Mitigation: derive marker values directly from documented snapshot values and assert cross-equality.
- Risk: overfitting to current tests while missing README contracts.
  - Mitigation: assert all contract key families in new R53 docs-contract test.
- Risk: spec-cap regression from new lifecycle dir.
  - Mitigation: compensating archive cleanup and archive policy checker.

## Interfaces / Contracts
New test contract: `review_r53_docs_contract.rs` enforces R53 marker completeness and core invariants.

## ADR
No ADR required (docs/test/governance surface only).
