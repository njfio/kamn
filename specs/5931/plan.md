# Plan: Issue #5931 - Task: Harden managed signer execution and secret env handling

- Issue: #5931
- Spec: `specs/5931/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: added managed signer security regressions for shell-injection payload execution and signer-secret env leakage to `crates/kamn-node/src/signer/managed_backend.rs`.
2. Implemented argv-tokenized command parsing and direct `Command::new` spawn path, removing shell interpolation from managed signer backend execution.
3. Implemented child-process env scrubbing (`env_clear`) with explicit allowlist pass-through plus required signer request context markers.
4. REGRESSION: ran managed-backend unit tests and signer integration matrix (`main_tests::signer_tests`).
5. VERIFY: ran `cargo fmt --check`, strict `cargo clippy -p kamn-node --bin kamn-node -- -D warnings`, and signer/doc contract slices.

## Affected Modules (Initial)
- `crates/kamn-node/src/signer/managed_backend.rs`
- `docs/architecture/signer-lifecycle.md`
- `docs/foundation/node-runtime-cli.md`
- `docs/foundation/kolme-runtime-commit-client.md`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5931/spec.md`.
- Upstream issue contract: GitHub issue #5931.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- Not required (no new dependency or wire-format/protocol change).
