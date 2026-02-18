# Issue #4960 Tasks

- Issue: #4960
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing tests derived from issue ACs and conformance cases.
- [x] T2 (Green): implement minimum change to satisfy tests deterministically.
- [x] T3 (Refactor): simplify and harden without changing behavior.
- [x] T4 (Regression): add drift/tamper/marker parity regression checks.
- [x] T5 (Docs): update required docs/process markers for issue #4960.
- [x] T6 (Verify): run scoped unit/functional/integration/regression checks and record evidence.

## Completion Evidence
- Red:
  - `bash scripts/ci/test_check_stale_script_references.sh`
    - failed in early detector wiring with missing executable wrapper contract.
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
    - failed before ci-fast-gate stale-reference wiring with:
      `expected stale script reference checker step in ci-fast-gate workflow`
  - `cargo test -p kamn-core checklist_contains_stale_script_reference_deletion_wave_gate -- --exact`
    - failed before checklist marker landing with missing:
      `## Stale Script Reference Deletion-Wave Gate (Issues #4960, #4972)`
- Green/Regression:
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/check_stale_script_references.sh --output-json /tmp/stale-script-reference-report.json`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
  - `cargo test -p kamn-core checklist_contains_stale_script_reference_deletion_wave_gate -- --exact`
- Linked implementation PRs:
  - `#4981` (`#4971`) detector tests and command-surface integration
  - `#4982` (`#4972`) ci-fast-gate wiring and release go/no-go marker contracts
