# Plan: Issue #5785 — Finalize R53 Milestone Closure Markers and Docs-Contract Guard

- Issue: #5785
- Spec: `specs/5785/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED docs-contract assertion(s) in existing R53 docs-contract module to require closed milestone markers.
2. Run targeted test lane to capture failing evidence.
3. Update `specs/milestones/r53-e2e-scenario-execution-activation/index.md` to closure-complete state.
4. Re-run targeted and group-level docs-contract tests to GREEN.
5. Run formatting and scoped clippy/test checks.
6. Update active milestone (`r52`) index with new completed slice entry for `#5785`.

## Affected Modules
- `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs`
- `specs/milestones/r53-e2e-scenario-execution-activation/index.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5785/{spec.md,plan.md,tasks.md}`

## Risks / Mitigations
- Risk: spec-directory non-regression cap (693) may be exceeded by adding `specs/5785/`.
  - Mitigation: delete one obsolete archived pointer-only spec directory in the same change set.
- Risk: docs-contract assertions may be too strict for minor wording variation.
  - Mitigation: assert stable contract phrases already used across milestone indexes.

## Interfaces / Contracts
- Milestone index contract for closed slices: `Active issue(s): None` and slice status marker `(Completed)`.
- Existing issue-ID reference contract remains unchanged.

## ADR
- None required (no dependency/protocol/architecture decision change).
