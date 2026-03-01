# Spec: Issue #6291 - Types canonical DID parse helpers + integration lane

## Objective

Add canonical DID parse helpers to `kamn-types` that normalize surrounding whitespace and expose
typed error semantics, plus first crate-level integration coverage.

## Inputs/Outputs

- Inputs:
  - DID string input (`&str`) from shared crate consumers.
- Outputs:
  - `parse_agent_did_canonical`: `Result<AgentDid, SharedDidParseError>`
  - `parse_kamn_did_canonical`: `Result<KamnDid, SharedDidParseError>`

## Boundaries/Non-goals

- In scope:
  - Add helper APIs and typed error enum in `crates/kamn-types/src/lib.rs`.
  - Add integration tests in `crates/kamn-types/tests/`.
- Out of scope:
  - `kamn-core` parser logic changes.
  - DID format redesign.

## Failure Modes

- FM-1: whitespace-only input is accepted or maps ambiguously.
- FM-2: helper APIs lose source parser error detail.
- FM-3: helper APIs diverge between agent and generic KAMN DID paths.

## Acceptance Criteria

- AC-1: `parse_agent_did_canonical` trims surrounding whitespace and fails with
  `SharedDidParseError::EmptyInput` when empty.
- AC-2: `parse_kamn_did_canonical` trims surrounding whitespace and fails with
  `SharedDidParseError::EmptyInput` when empty.
- AC-3: source parser failures are preserved via `SharedDidParseError::{Agent,Kamn}` wrappers.
- AC-4: integration tests cover success and typed error paths for both helpers.
- AC-5: existing `kamn-types` tests remain green.

## Files To Touch

- `crates/kamn-types/src/lib.rs`
- `crates/kamn-types/tests/canonical_did_parse_integration.rs`
- `specs/6291-types-canonical-did-parse-helpers.md`

## Error Semantics

- New typed error:
  - `EmptyInput`
  - `Agent(AgentDidError)`
  - `Kamn(KamnDidError)`
- No silent fallback; normalization is explicit trim-only.

## Test Plan

- RED:
  - Add integration tests referencing new helper APIs + error enum.
  - Confirm tests fail before implementation.
- GREEN:
  - Implement helper functions and error enum.
- REFACTOR:
  - Add `Display`/`Error` for wrapper enum.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-types --tests -- -D warnings`
  - `cargo test -p kamn-types`
