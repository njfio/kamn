# Issue #3938 Plan

- Issue: #3938
- Status: Completed
- Spec: `specs/3938/spec.md`

## Implementation Approach
1. Execute extraction subtask `#3944` to split `runtime_tests.rs` into focused include fragments and add bounded-shell contracts.
2. Execute parity subtask `#3945` to add deterministic command-surface selector/docs checks.
3. Validate with targeted and contract regression runs.
4. Close task with child linkage and evidence markers.

## Affected Modules
- `crates/kamn-node/src/main_tests/runtime_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/*.rs`
- `crates/kamn-node/tests/main_module_extraction_contract.rs`
- `crates/kamn-node/tests/main_tests_command_surface_parity_contract.rs`
- `docs/ci/strategy.md`
- `docs/foundation/runtime-watchdog-attestation.md`

## Risks and Mitigations
- Risk: extraction could break selector discoverability.
  - Mitigation: parity contract requires explicit selector symbol and command markers.
- Risk: decomposition could regress into inline monolith.
  - Mitigation: extraction contract enforces bounded shell/no-inline-tests marker.

## Contracts and Interfaces
- Runtime selector namespace remains `main_tests::runtime_tests::<selector>`.
- Docs parity markers remain deterministic and fail closed when drifted.

## Verification Strategy
- RED/GREEN/REGRESSION evidence captured in merged PRs #5160 and #5161.
