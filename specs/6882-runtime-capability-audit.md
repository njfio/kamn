# 6882 Runtime Capability Audit

## Objective
Establish a spec-backed, evidence-backed audit of KAMN runtime capability for message routing, task dispatch, audit emission/export, and live transport so repository claims reflect actual current behavior on `main`.

## Inputs/Outputs
- Inputs:
  - Current `main` source code under `crates/kamn-core`, `crates/kamn-node`, `crates/kamn-sdk`, and `crates/kamn-e2e-harness`
  - Existing tests, runtime scripts, and workflow evidence
- Outputs:
  - A committed runtime capability audit document with explicit per-path status
  - A hard-fail docs contract test that validates required audited sections and status markers
  - Follow-on issue links for missing or partial capability where appropriate

## Boundaries/Non-goals
- Do not implement missing runtime features in this issue
- Do not rewrite the README wholesale
- Do not redesign architecture beyond documenting actual current path status

## Failure modes
- Audit overstates capability without executable evidence
- Audit understates capability by ignoring wired runtime/test paths
- Audit uses ambiguous labels instead of explicit status buckets
- Follow-on gaps are identified but not linked to issues

## Acceptance criteria
- [ ] A runtime capability audit document exists and is committed
- [ ] The audit covers message routing, task dispatch, audit emission/export, and live transport
- [ ] Each area is classified as implemented_and_wired, gated_or_partial, contract_only, or missing
- [ ] Each area includes at least one concrete evidence point (test, script, or entrypoint)
- [ ] The audit links known missing/partial paths to issue IDs where available
- [ ] A docs contract test validates the audit structure and required markers

## Files to touch
- `specs/6882-runtime-capability-audit.md`
- `docs/review/runtime-capability-audit-r57.md`
- `crates/kamn-core/tests/runtime_capability_audit_docs.rs`

## Error semantics
- Missing required audit sections or invalid status markers fail the contract test
- Claims without evidence markers are treated as invalid audit output

## Test plan
- Red: add `runtime_capability_audit_docs` contract expecting the new audit doc and required markers; confirm failure
- Green: write the audit doc with required sections, statuses, evidence markers, and issue links; confirm contract passes
- Integration: verify cited commands/tests exist and are callable from current repo paths
