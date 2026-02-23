# Plan: Issue #5810 - Spec-Volume Cap Reconciliation

- Issue: #5810
- Status: Completed
- Spec: `specs/5810/spec.md`

## Approach
1. Capture RED evidence from `review_r53_docs_contract` showing cap assertion failure.
2. Remove the minimum required legacy top-level archived-spec pointer directories (`specs/3926`, `specs/3925`) while retaining canonical archived content under `specs/archive/`.
3. Re-run docs-contract lane to confirm guardrail restoration.
4. Re-run `kamn-e2e-harness` regression lane to ensure no functional drift from `#5808`.
5. Finalize milestone and lifecycle artifacts.

## Affected Artifacts
- `specs/3926/` (top-level archived pointer directory removal)
- `specs/3925/` (top-level archived pointer directory removal)
- `specs/5810/spec.md`
- `specs/5810/plan.md`
- `specs/5810/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: removing a spec pointer that is still referenced.
  - Mitigation: target an already archived pointer-only directory whose canonical spec remains in `specs/archive/3926`.
- Risk: hidden regressions from late-cycle remediation.
  - Mitigation: rerun failing docs-contract lane plus harness regression lane.

## Verification Strategy
- RED: failing `review_r53_docs_contract` assertion.
- GREEN: same lane passes post-remediation.
- Regression: full `kamn-e2e-harness` crate tests pass.
