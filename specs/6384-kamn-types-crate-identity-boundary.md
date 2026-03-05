# Spec: Issue #6384 - Resolve kamn-types crate identity boundary

## Objective

Resolve `kamn-types` crate identity ambiguity by establishing an explicit DID-focused boundary with stable import compatibility and documented migration guidance.

## Inputs/Outputs

- Inputs:
  - issue `#6384`
  - existing crate surface in `crates/kamn-types/src/lib.rs`
  - crate docs in `crates/kamn-types/README.md` and `docs/architecture/kamn-types.md`
- Outputs:
  - explicit `kamn_types::did` identity boundary with stable top-level compatibility exports
  - docs markers declaring crate ownership scope and migration guidance
  - contract tests enforcing boundary and docs markers

## Boundaries/Non-goals

- In scope:
  - define and implement crate direction as a DID helper boundary (not broad pass-through)
  - preserve existing top-level imports used by in-workspace consumers
  - add docs/contract tests for identity boundary
- Out of scope:
  - unrelated type-system redesign
  - behavior changes to DID parsing semantics
  - silent API breakage for existing `kamn-types` imports

## Failure modes

- FM-1: crate remains effectively pass-through with no explicit identity boundary marker.
- FM-2: migration guidance drifts from actual exported surfaces.
- FM-3: boundary refactor breaks existing top-level imports used by downstream crates.

## Acceptance criteria (testable booleans)

- [x] AC-1: target direction is documented as an explicit DID-focused `kamn-types` identity boundary.
- [x] AC-2: chosen implementation preserves stable imports and does not change DID parse behavior.
- [x] AC-3: README and architecture docs include deterministic migration/ownership markers.
- [x] AC-4: contract tests validate boundary exports and docs markers.

## Files to touch

- `specs/6384-kamn-types-crate-identity-boundary.md`
- `crates/kamn-types/src/lib.rs`
- `crates/kamn-types/README.md`
- `docs/architecture/kamn-types.md`
- `crates/kamn-types/tests/canonical_did_parse_integration.rs`
- `crates/kamn-types/tests/identity_boundary_contract.rs` (new)

## Error semantics

- Existing parse error propagation remains fail-closed and typed.
- Boundary changes must not swallow underlying parse errors.
- Any docs-contract drift must fail tests deterministically.

## Test plan

- RED:
  - add contract tests that require explicit identity/migration markers in docs and `did` boundary exports.
  - verify new tests fail before implementation/docs updates.
- GREEN:
  - implement `did` boundary module and compatibility re-exports.
  - update README/architecture docs with deterministic markers and migration guidance.
- REFACTOR:
  - remove duplicate export wiring and keep single-source boundary definitions.
- INTEGRATION:
  - run `cargo test -p kamn-types`
  - run dependent targeted suites that import `kamn-types` (`kamn-sdk`, `kamn-agent-lib`)

## Phase 6 integration evidence

- 2026-03-05: `cargo test -p kamn-types --test identity_boundary_contract` (pass)
- 2026-03-05: `cargo test -p kamn-types` (pass)
- 2026-03-05: `cargo test -p kamn-agent-lib` (pass)
- 2026-03-05: `cargo test -p kamn-sdk` (pass)

## Deviations

- None.
