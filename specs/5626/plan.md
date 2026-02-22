# Plan: #5626 Mode Execution Contract Markers

## Approach
1. Compute deterministic mode-driver marker from execution mode.
2. Emit `mode_execution_contract` JSON object using selected and executed scenario counts.
3. Add command-contract tests for object presence, mode-driver mapping, and count parity.
4. Keep existing contract objects stable and rerun full crate tests.
5. Add docs artifact + docs contract test and refresh milestone index progress markers.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/` (R53 mode execution artifact)
- `crates/kamn-e2e-harness/tests/` (new docs contract test)
- `specs/milestones/r53-e2e-scenario-execution-activation/index.md`

## Risks and Mitigations
- Risk: output contract expansion may break strict string assertions.
  - Mitigation: deterministic ordering and focused contract tests.
- Risk: mode-driver mapping drift over time.
  - Mitigation: explicit tests across three representative modes.

## Interfaces / Contracts
- New field:
  - `mode_execution_contract:{"mode":"...","driver":"...","selected_scenarios":u64,"executed_scenarios":u64,"status":"PASS|FAIL"}`

## ADR
- Not required.
