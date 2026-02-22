# Plan: #5688 Decompose `kamn-e2e-harness` Run-Contract Monolith

## Approach
1. Capture baseline for `lib.rs` size and run-contract test pass state.
2. Create a new internal module (e.g., `run_contract.rs`) containing:
   - `execute_run_contract`
   - runtime probe helpers
   - lifecycle aggregation and phase-detail helpers
3. Keep root exports stable from `lib.rs` (`pub use` or wrapper).
4. Adjust internal visibility only as needed for existing unit tests.
5. Run targeted harness suites and format/lint checks.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/run_contract.rs` (new)

## Risks and Mitigations
- Risk: subtle JSON output drift from refactor.
- Mitigation: rely on existing contract suites with exact marker assertions.

- Risk: private helper visibility breaks unit tests.
- Mitigation: expose minimal `pub(crate)` helpers only where required.

## Interfaces / Contracts
- Public API remains unchanged:
  - `execute_run_contract(&RunCommandConfig) -> Result<String, String>`
- Deterministic run output schema remains unchanged.

## ADR
- Not required. No protocol or dependency change.
