# Plan: Issue #5923 - Task: Replace deterministic agent-lib auth signatures with cryptographic signatures

- Issue: #5923
- Spec: `specs/5923/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: Added `auth_roundtrip` conformance tests that failed against deterministic signing path:
   - cryptographic contract equality mismatch,
   - non-private-key signing material unexpectedly accepted,
   - deterministic forgery collision,
   - same-length tamper collision.
2. Implemented cryptographic request-auth signing in `kamn-agent-lib/src/auth.rs` via new SDK helper `service_signature_for_state_hash_with_private_key`.
3. Added SDK helper in `kamn-sdk/src/service.rs` (and re-export in `kamn-sdk/src/lib.rs`) to sign canonical service-auth payloads from explicit state hash + private key.
4. REGRESSION/VERIFY: Ran targeted agent-lib and sdk suites plus format and strict clippy across touched crates.

## Affected Modules (Initial)
- `crates/kamn-agent-lib/src/auth.rs`
- `crates/kamn-agent-lib/tests/auth_roundtrip.rs`
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/src/lib.rs`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5923/spec.md`.
- Upstream issue contract: GitHub issue #5923.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- ADR required if this issue introduces a new dependency, protocol/wire-format change, or architecture boundary change.

## Verification Commands
- `cargo test -p kamn-agent-lib --test auth_roundtrip -- --nocapture`
- `cargo test -p kamn-agent-lib -- --nocapture`
- `cargo test -p kamn-sdk -- --nocapture`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features --manifest-path crates/kamn-agent-lib/Cargo.toml -- -D warnings`
- `cargo clippy --all-targets --all-features --manifest-path crates/kamn-sdk/Cargo.toml -- -D warnings`
