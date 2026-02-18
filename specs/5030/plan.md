# Issue #5030 Plan

- Issue: #5030
- Status: Implemented

## Approach
1. Add RED tests for:
   - deterministic proof decision reason constants (valid/invalid),
   - anchoring failure-matrix stable/drift outcomes,
   - fail-closed invalid matrix inputs.
2. Implement additive M1 contracts:
   - proof decision enum + reason constants,
   - anchoring failure-matrix case/evidence/report contracts,
   - failure-matrix evaluation API with typed invalid-input errors.
3. Export new M1 symbols through `kamn-core` crate root and preserve existing
   M1 behavior.
4. Run scoped/full regression gates plus shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m1.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m1_merkle_anchoring.rs`
- `specs/5030/spec.md`
- `specs/5030/plan.md`
- `specs/5030/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing merkle/proof/anchoring behavior.
  - Preserve matrix evidence input order for deterministic drift reporting.
  - Keep work Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive API/exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
