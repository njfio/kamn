# Issue #5031 Plan

- Issue: #5031
- Status: Implemented

## Approach
1. Add RED tests for negative-matrix `AllDenied`/`DriftDetected` paths and
   ABAC reason-constant assertions.
2. Implement additive M2 negative-matrix contracts:
   `DataLayerM2NegativeAuthorizationCase`,
   `DataLayerM2NegativeAuthorizationMatrixDecision`,
   `DataLayerM2NegativeAuthorizationMatrixReport`, and
   `DataLayerM2AbacEngine::evaluate_negative_authorization_matrix(...)`.
3. Export stable ABAC reason constants and remove string literals from M2 code
   and tests.
4. Run scoped/full regression gates plus shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m2_gateway_access.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m2_gateway_access.rs`
- `specs/5031/spec.md`
- `specs/5031/plan.md`
- `specs/5031/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing auth/ABAC behavior.
  - Keep audit fixtures deterministic by preserving input case order.
  - Keep work Rust-only to guarantee `shell_loc_delta_actual = 0`.

## Interface Contract
- Additive API and exports in `kamn-core`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
