# Issue #5004 Plan

- Issue: #5004
- Status: Implemented

## Approach
1. Deliver M1 trust-anchor contracts via `#5017`:
   - deterministic merkle batch assembly,
   - inclusion-proof generation and fail-closed verification,
   - idempotent Kolme anchoring worker behavior.
2. Deliver M1 conformance expansion via `#5030`:
   - proof-verification decision reason markers,
   - anchoring failure-matrix stable/drift evidence contracts,
   - fail-closed matrix input handling.
3. Maintain additive exports in `kamn-core` and preserve shell-neutral delivery.
4. Validate with scoped and crate-level tests plus shell guardrail evidence.

## Affected Modules
- `crates/kamn-core/src/data_layer_m1.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_merkle_anchoring.rs`
- `specs/5004/spec.md`
- `specs/5004/plan.md`
- `specs/5004/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep implementation additive and deterministic.
  - Preserve fail-closed behavior for invalid proofs and matrix inputs.
  - Keep shell/workflow surfaces unchanged to maintain ratio guardrails.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this bounded additive story closure.
