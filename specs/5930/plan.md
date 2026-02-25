# Plan: Issue #5930 - Task: Implement HTTPS support in SDK service client and TLS validation

- Issue: #5930
- Spec: `specs/5930/spec.md`
- Status: Implemented
- Last Updated: 2026-02-25

## Approach
1. RED: added HTTPS fixture-conformance tests to `crates/kamn-sdk/tests/service_api_client.rs` and confirmed failure on the legacy `NotImplemented` path.
2. Implemented rustls-backed HTTPS transport in `crates/kamn-sdk/src/service.rs` with strict certificate validation and deterministic TLS error mapping.
3. Added explicit custom trust-root control via `KAMN_SERVICE_API_TLS_CA_FILE` (with fail-closed validation for empty/invalid/missing CA bundle inputs).
4. REGRESSION: ran full `kamn-sdk` test suite and scoped service-client contract suite.
5. VERIFY: ran `cargo fmt --check` and strict `cargo clippy -p kamn-sdk -- -D warnings`.

## Affected Modules (Initial)
- `crates/kamn-sdk/src/service.rs`
- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-sdk/Cargo.toml`

## Risks + Mitigations
- Risk: Scope expansion across multiple crates can increase merge and verification time.
  - Mitigation: keep PRs task-scoped and verify each AC with targeted tests before running broader suites.
- Risk: Security/runtime changes can regress existing contracts.
  - Mitigation: preserve and extend regression coverage before behavior changes.
- Risk: Cross-issue dependencies can block downstream tasks.
  - Mitigation: execute in dependency order and keep blockers logged in issue comments.

## Interfaces / Contracts
- Primary contract source: `specs/5930/spec.md`.
- Upstream issue contract: GitHub issue #5930.
- Protocol/API/schema changes require explicit documentation updates and linked follow-up issues when out of scope.

## ADR Requirement
- Required and satisfied: `docs/architecture/adr-kamn-sdk-service-https-transport.md`.
