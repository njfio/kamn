# Plan: Issue #5911 - Disable Legacy Baseline-v1 Signature Compatibility in Production Builds

## Approach
1. Harden `signer_legacy_baseline_v1_compat_enabled` in both `signer_backend.rs` and `transaction.rs` to return `false` outside debug assertions.
2. Keep existing env parsing for debug/test branch to preserve compatibility fixtures.
3. Add targeted helper tests to lock branch behavior semantics where practical.
4. Run signer backend and transaction guard test slices plus fmt/clippy/mutation checks.

## Affected Modules
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-core/src/transaction.rs`

## Risks / Mitigations
- Risk: Existing compatibility tests rely on env behavior.
  - Mitigation: preserve env behavior in debug/test branch and keep regression suite coverage.
- Risk: policy drift between signer and transaction helpers.
  - Mitigation: apply same hardening pattern in both modules and verify with targeted tests.

## Interfaces / Contracts
- No wire/API changes.
- Runtime policy contract tightens: legacy baseline-v1 compatibility is never available in non-debug builds.

## ADR
- Not required (no dependency/protocol/architecture boundary change).
