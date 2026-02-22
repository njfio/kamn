# Issue #5608 Plan - R51 Milestone Closeout Marker Finalization

## Approach
1. Add RED docs-contract assertions for closeout markers.
2. Update milestone index markers to closed/completed state.
3. Rerun docs-contract test to GREEN.

## Affected Modules
- `crates/kamn-e2e-harness/tests/phase6_runtime_validation_docs_contract.rs`
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: stale marker semantics post-merge.
  - Mitigation: explicit assertions for completed/active/line-state markers.

## Interfaces / Contracts
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md` marker lines.

## ADR
- Not required (docs marker closeout only).
