# Spec: Issue #6021 - Add M2 gateway-access invariant unit tests

- Issue: #6021
- Status: Implemented
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26

## Problem Statement
`crates/kamn-core/src/data_layer_m2_gateway_access.rs` currently has no direct unit tests despite implementing append-only access-audit hash chains and negative-authorization matrix drift reporting.

## Scope
In scope:
- Add direct `#[cfg(test)]` coverage in `data_layer_m2_gateway_access.rs`.
- Validate deterministic access-audit append/hash-chain verification behavior.
- Validate tamper-path failure when lineage hash links are mutated.
- Validate negative-authorization matrix stable/drift decision projection.

Out of scope:
- Service API wiring changes.
- M3+ data-layer module testing.

## Risk Level
`medium`

## Acceptance Criteria
- AC-1: Access-audit append behavior directly validates deterministic sequence and hash-chain linkage.
- AC-2: Tampered audit lineage fails closed with deterministic `InvalidAuditHashChain` errors.
- AC-3: Negative-authorization matrix emits stable decision when all deny expectations match and drift decision when any expectation mismatches.

## Conformance Cases
- C-01 (Unit, AC-1): append two audit records and verify deterministic sequence, genesis link, and chain verification success.
- C-02 (Regression, AC-2): tamper a stored record hash and verify chain validation returns `InvalidAuditHashChain`.
- C-03 (Unit, AC-3): evaluate matrix cases that produce `AllDenied` and `DriftDetected` decisions with deterministic fixture evidence.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core data_layer_m2_gateway_access -- --nocapture` passes with new direct tests.
