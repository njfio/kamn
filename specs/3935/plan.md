# Issue #3935 Plan

- Issue: #3935
- Status: Completed
- Spec: `specs/3935/spec.md`

## Implementation Approach
1. Complete decomposition task `#3938`.
2. Complete governance task `#3939`.
3. Validate with extraction, parity, budget, and ownership docs contracts.

## Affected Modules
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `crates/kamn-node/tests/main_tests_surface_budget_contract.rs`
- `crates/kamn-core/tests/node_test_surface_ownership_docs.rs`
- `docs/ci/strategy.md`
- `docs/foundation/runtime-watchdog-attestation.md`

## Verification Strategy
- RED/GREEN/REGRESSION evidence in merged PRs #5160, #5161, #5163, #5164 and task closeout #5162.
