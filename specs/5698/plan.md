# Plan: #5698 Activate Opt-In Live CLI-Scripted S-01 Driver Execution

## Approach
1. Refactor `CliScriptedDriver` into configurable driver with:
   - live toggle flag
   - injectable command runner for testability
2. Implement default runner that executes:
   - `kamn-cli health --endpoint <KAMN_ENDPOINT> --format text`
3. Keep deterministic pass path as default when live mode toggle is disabled.
4. Update run-contract mode driver creation to use `CliScriptedDriver::from_env`.
5. Add conformance tests for toggle/runner execution mapping and run full harness
   regression suite.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/run_contract.rs`
- `crates/kamn-e2e-harness/tests/*` (new conformance tests)

## Risks and Mitigations
- Risk: live command path may fail due missing binary in live mode.
  Mitigation: failures map to scenario fail only in opt-in mode; default mode is
  unchanged deterministic pass.

- Risk: output contract regressions.
  Mitigation: run full harness contract suites and keep run JSON schema unchanged.

## Interfaces / Contracts
- `HarnessDriver` trait remains unchanged.
- `CliScriptedDriver` exposes configurable constructor for tests and env-based
  default constructor for runtime.

## ADR
- Not required: no dependency or external protocol changes.
