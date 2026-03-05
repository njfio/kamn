# Spec: Issue #6417 - Add deterministic specs index and docs contract guard

## Objective

Add a deterministic top-level specs index at `specs/INDEX.md` and enforce its required navigation markers through a docs contract test so spec discovery does not regress.

## Inputs/Outputs

- Inputs:
  - current `specs/` tree with no top-level index
  - docs-contract testing pattern under `crates/kamn-core/tests`
- Outputs:
  - `specs/INDEX.md` with stable section markers and curated links
  - `crates/kamn-core/tests/specs_index_docs.rs` enforcing required index markers

## Boundaries/Non-goals

- In scope:
  - deterministic index content and enforceable marker contract
  - curated links for currently active/high-priority work tracks
- Out of scope:
  - exhaustive catalog of every historical spec
  - automatic generation tooling
  - modifying existing spec internals

## Failure modes

- FM-1: index file missing.
- FM-2: index exists but required navigation markers are absent.
- FM-3: docs contract test missing, allowing silent drift.
- FM-4: docs contract lane fails.

## Acceptance criteria (testable booleans)

- [x] AC-1: `specs/INDEX.md` exists.
- [x] AC-2: index includes deterministic markers for purpose, naming convention, status taxonomy, and curated links.
- [x] AC-3: docs contract test checks required index markers and fails closed when they regress.
- [x] AC-4: `cargo test -p kamn-core --test specs_index_docs` passes.

## Files to touch

- `specs/6417-specs-index-docs-contract.md`
- `specs/INDEX.md`
- `crates/kamn-core/tests/specs_index_docs.rs`

## Error semantics

- Test-only fail-closed behavior via assertion failures on missing index markers.

## Test plan

- RED:
  - add `specs_index_docs` contract test asserting required `INDEX.md` markers.
  - run test; confirm fail because index file is absent.
- GREEN:
  - add `specs/INDEX.md` with required markers and curated links.
  - rerun test; confirm pass.
- REFACTOR:
  - keep marker keys concise and deterministic.
- INTEGRATION:
  - run docs index contract lane and record evidence.

## Phase 6 integration evidence

- `cargo test -p kamn-core --test specs_index_docs` -> PASS (`1 passed, 0 failed`)

## Deviations

- None.
