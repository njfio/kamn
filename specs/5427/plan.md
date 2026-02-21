# Issue #5427 Plan — Deterministic Batching Policy for Coherence Hardening

## Approach
1. Add a new docs contract test file validating coherence-batching policy markers in `gaps-and-issues-r45.md`.
2. Run RED against current docs (markers absent).
3. Update review doc with policy section + deterministic markers.
4. Run GREEN targeted docs tests + fmt/clippy gates.

## Affected Modules
- `docs/review/gaps-and-issues-r45.md`
- `crates/kamn-core/tests/review_coherence_batching_policy_docs_contract.rs` (new)
- `specs/5427/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: policy markers become stale/non-actionable.
  - Mitigation: enforce numeric consistency in docs contract tests.
- Risk: accidental shell/process surface churn.
  - Mitigation: constrain scope to docs + Rust test file only.

## Interfaces / Contracts
- Policy markers must be deterministic key/value lines under one section.
- Marker schema version and numeric targets must parse cleanly and satisfy consistency rules.

## Validation Strategy
- RED: new docs contract test fails before marker insertion.
- GREEN: docs update makes new tests pass.
- VERIFY: run targeted docs tests, fmt, clippy.
