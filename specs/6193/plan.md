# Plan: Issue 6193 - Signer Adapter Must Not Clone Private Key Material

- Issue: #6193
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Remove `Clone` derive from the signer adapter struct.
2. Add a boundary-contract assertion to fail if clone derive reappears.
3. Run signer-focused and boundary-contract test lanes.

## Affected Modules

- `crates/kamn-node/src/signer/signer_adapter.rs`
- `crates/kamn-node/tests/signer_adapter_boundary_contract.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs` (verification lane)

## Risks and Mitigations

1. Risk: hidden call sites depend on adapter cloning.
   - Mitigation: compile and signer test lane validation.
2. Risk: future regressions reintroduce clone derive.
   - Mitigation: explicit boundary-contract source assertion.
