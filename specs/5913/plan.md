# Plan: Issue #5913 - Remove Compiled Fallback Signer Key Paths From kamn-core

## Approach
1. Remove fallback key resolution calls from:
   - `crates/kamn-core/src/signer_backend.rs`
   - `crates/kamn-core/src/transaction.rs`
2. Keep existing explicit env precedence unchanged.
3. Add/adjust regression tests for missing-key fail-closed behavior.
4. Update integration tests to provide explicit signer key env for local software fallback lanes that currently rely on implicit fallback.
5. Verify with targeted test lanes, fmt/clippy, and in-diff mutation.

## Affected Modules
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-core/src/transaction.rs`
- `crates/kamn-core/tests/signer_backend.rs` (if explicit env setup needed for local-fallback tests)

## Risks / Mitigations
- Risk: existing tests implicitly depend on fallback key behavior.
  - Mitigation: add explicit test env guards/fixtures for local fallback tests.
- Risk: accidental behavior drift in secure-provider paths.
  - Mitigation: keep secure-provider logic unchanged and run signer backend integration lane.

## Interfaces / Contracts
- No public API or wire-format changes.
- Runtime policy contract tightens to explicit key provisioning only.

## ADR
- Not required (no dependency/protocol/architecture boundary change).
