# Issue #5005 Plan

- Issue: #5005
- Status: Implemented

## Approach
1. Deliver M2 gateway contracts through child task `#5018`:
   - DID auth/session issuance with deterministic identifiers and bounded TTL.
   - Fail-closed ABAC message visibility engine with stable reason taxonomy.
   - Deterministic negative-authorization matrix drift detection.
   - Static RLS policy template contracts and append-only audit hash-chain checks.
2. Preserve additive exports in `kamn-core` for downstream integration.
3. Validate with scoped suite `data_layer_m2_gateway_access` plus crate-level regression.
4. Keep delivery Rust-only for this story to preserve shell budget neutrality.

## Affected Modules
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs`
- `specs/5005/spec.md`
- `specs/5005/plan.md`
- `specs/5005/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Keep fail-closed semantics explicit across authn/authz negative paths.
  - Validate deterministic reason markers via conformance tests.
  - Preserve rust-only implementation to avoid shell-surface growth.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
