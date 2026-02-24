# Plan: Issue #5909 - Fail Closed Insecure Deterministic Message Crypto by Default

## Approach
1. Remove `cfg!(debug_assertions)` auto-enable branches in direct/group crypto env gate helpers.
2. Keep env-based opt-in parser logic unchanged (`1|true|yes|on`).
3. Add RED tests proving constructors fail closed when env is unset.
4. Add/confirm tests for explicit opt-in success paths.

## Affected Modules
- `crates/kamn-core/src/direct_message_crypto.rs`
- `crates/kamn-core/src/group_channel_crypto.rs`

## Risks / Mitigations
- Risk: Existing local tests relying on implicit debug enable may fail.
  - Mitigation: update tests to set/unset env deterministically using process-level synchronization patterns already used in this crate.
- Risk: Env-state leakage between tests.
  - Mitigation: use scoped env guards in tests and avoid parallel mutation hazards.

## Interfaces / Contracts
- Constructor behavior contract tightens: no implicit debug enable; explicit env opt-in required.
- Error contract remains deterministic: `InsecureCryptoDisabled` when opt-in absent.

## ADR
- Not required (no dependency/protocol/architecture boundary change).
