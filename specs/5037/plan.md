# Issue #5037 Plan

- Issue: #5037
- Status: Implemented

## Approach
1. Add RED conformance test for duplicate wrapped-key recipient rejection in M8
   registration flow.
2. Implement fail-closed wrapped-key recipient uniqueness validation with typed
   error output.
3. Keep existing crypto-shred/retention/legal-hold conformance paths stable.
4. Run scoped/full regression and shell guardrail evidence commands.

## Affected Modules
- `crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs`
- `crates/kamn-core/tests/data_layer_m8_compliance_lifecycle.rs`
- `specs/5037/spec.md`
- `specs/5037/plan.md`
- `specs/5037/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Mitigations:
  - Keep validation additive and fail-closed.
  - Preserve existing reason-marker and legal-hold behavior.
  - Keep implementation Rust-only; no shell/workflow changes.

## Interface Contract
- Additive error taxonomy entry for duplicate wrapped-key recipient validation.
- No dependency/protocol/wire-format changes.

## ADR
- Not required for this scoped additive validation contract.
