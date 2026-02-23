# Tasks: Issue #5810 - Spec-Volume Cap Reconciliation

- Issue: #5810
- Spec: `specs/5810/spec.md`
- Plan: `specs/5810/plan.md`
- Status: Done

## Ordered Tasks
- [x] T1 (RED/Conformance): reproduce cap regression via `review_r53_docs_contract`.
- [x] T2 (GREEN): remove the minimum required legacy archived top-level spec pointer directories to restore cap compliance.
- [x] T3 (Regression): rerun docs-contract and harness lanes.
- [x] T4 (Closeout): update milestone/lifecycle markers to completed state.

## Tier Mapping
- Conformance: pre/post docs-contract cap lane.
- Functional: cap restoration through bounded spec-pointer deletion.
- Regression: full `kamn-e2e-harness` test lane.
- Performance: N/A (no runtime path changes).
