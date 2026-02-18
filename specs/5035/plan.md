# Issue #5035 Plan

- Issue: #5035
- Status: Implemented

## Approach
1. Add RED conformance tests for scoped portability export authorization and
   parity with existing owner projection output.
2. Implement additive scoped portability export API and reason-marker constants
   for owner-scope/cross-owner denial paths.
3. Replace string-literal reason checks with exported constants in tests.
4. Run scoped/full regression and shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m6_graph_integration.rs`
- `crates/kamn-core/tests/data_layer_m6_graph_integration.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5035/spec.md`
- `specs/5035/plan.md`
- `specs/5035/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep API additive and preserve existing projection behavior.
  - Use existing owner-scope error type with stable reason constants.
  - Keep implementation Rust-only; no shell/workflow changes.

## Interface Contract
- Additive API in `kamn_core::data_layer_m6_graph_integration`.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive contract.
