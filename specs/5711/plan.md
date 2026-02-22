# Plan: #5711 Reconcile R52 Post-Publication Quality-Gate Status Markers

## Approach
1. Extend an existing R52 docs-contract test suite to require:
   - README marker-schema template lines for quality-gate post-publication reconciliation.
   - R52 reconciliation markers and deterministic value constraints.
2. Run RED by executing the targeted test before doc updates.
3. Add schema template section to `docs/review/README.md`.
4. Add post-publication quality-gate reconciliation subsection + markers to
   `docs/review/gaps-and-issues-r52.md` without mutating as-of snapshot fields.
5. Re-run targeted tests and repository verification commands.

## Affected Modules
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r52.md`
- `crates/kamn-core/tests/review_r52_branch_hygiene_reconciliation_docs_contract.rs`

## Risks and Mitigations
- Risk: accidental mutation of historical R52 snapshot claims.
  - Mitigation: only append a dedicated post-publication subsection and assert snapshot lines remain.
- Risk: ratchet-driven collateral failures.
  - Mitigation: avoid adding a new docs-contract test file; extend existing R52 test file.

## Interfaces / Contracts
- Review marker schema contract in `docs/review/README.md`.
- R52 review artifact marker lines in `docs/review/gaps-and-issues-r52.md`.
- Docs-contract parser/validator in existing `kamn-core` test harness file.

## ADR
No ADR required (docs/contract reconciliation only; no architectural decision or dependency change).
