# Issue #5018 Plan

- Issue: #5018
- Status: Implemented

## Approach
1. Add red tests for C-01..C-06 in a dedicated `kamn-core` test suite:
   - DID auth/session issuance success/failure,
   - ABAC allow/deny matrix,
   - RLS policy contract rendering checks,
   - append-only audit hash-chain verification + tamper detection.
2. Implement `data_layer_m2_gateway_access` module with:
   - deterministic DID session token service,
   - ABAC message-scope authorizer,
   - static RLS policy template generator,
   - append-only access audit ledger.
3. Re-export M2 types/functions from `crates/kamn-core/src/lib.rs`.
4. Execute format/lint/scoped/full regression and finalize spec lifecycle markers.

## Affected Modules
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs` (new)
- `crates/kamn-core/src/lib.rs` (module + re-exports)
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs` (new)
- `specs/5018/spec.md`
- `specs/5018/plan.md`
- `specs/5018/tasks.md`

## Risks and Mitigations
- Risk level: high
- Risks:
  - Authorization drift between ABAC checks and generated RLS predicates.
  - Audit-chain design permitting silent mutation.
  - Session issuance accepting malformed identities.
- Mitigations:
  - Keep ABAC reason taxonomy explicit and test deny paths first.
  - Enforce append-only ledger semantics with hash-chain verification and tamper regression tests.
  - Validate DID and session TTL boundaries in constructor/auth methods.
  - Keep implementation Rust-only to preserve shell ratio constraints.

## Interface Contract
- Additive-only public API under `kamn_core::data_layer_m2_gateway_access::*`.
- No dependency additions.
- No protocol/wire-format changes.

## ADR
- Not required for this scoped additive implementation.
