# Plan: Issue #5804 - Add kamn-core Live Probe Matrix Module

- Issue: #5804
- Status: Completed
- Spec: `specs/5804/spec.md`

## Approach
1. Implement new module `live_probe_matrix` with:
   - typed mode enum
   - typed status enum
   - matrix entry + report structures
   - fail-closed validation (`duplicate mode/scenario`, empty scenario id)
   - deterministic aggregate helpers
2. Export module from `lib.rs`.
3. Add dedicated contract test file for happy/error/edge behavior.
4. Run targeted `kamn-core` tests and formatting gates.
5. Preserve spec-volume cap by offsetting lifecycle artifact addition with one legacy implemented spec prune.

## Affected Artifacts
- `crates/kamn-core/src/live_probe_matrix.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/live_probe_matrix_contract.rs`
- `specs/5804/spec.md`
- `specs/5804/plan.md`
- `specs/5804/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: over-scoped API surface.
  - Mitigation: keep module minimal, focused on validation + aggregation.
- Risk: missing edge behavior coverage.
  - Mitigation: explicit contract tests for duplicates/empty scenario ids and mixed outcomes.
- Risk: spec-cap non-regression breach.
  - Mitigation: prune one legacy implemented `specs/<id>/` directory in same PR.

## Verification Strategy
- Run test mapping commands from spec C-01..C-05.
- Confirm module count increment via `pub mod` inventory in `lib.rs`.
