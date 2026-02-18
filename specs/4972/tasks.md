# Issue #4972 Tasks

- Issue: #4972
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing tests derived from issue ACs and conformance cases.
- [x] T2 (Green): implement minimum change to satisfy tests deterministically.
- [x] T3 (Refactor): simplify and harden without changing behavior.
- [x] T4 (Regression): add drift/tamper/marker parity regression checks.
- [x] T5 (Docs): update required docs/process markers for issue #4972.
- [x] T6 (Verify): run scoped unit/functional/integration/regression checks and record evidence.

## Completion Evidence
- Red:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
    - failed with: `expected stale script reference checker step in ci-fast-gate workflow`
  - `cargo test -p kamn-core checklist_contains_stale_script_reference_deletion_wave_gate -- --exact`
    - failed with missing checklist section marker:
      `## Stale Script Reference Deletion-Wave Gate (Issues #4960, #4972)`
- Green/Regression:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `cargo test -p kamn-core checklist_contains_stale_script_reference_deletion_wave_gate -- --exact`
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/check_stale_script_references.sh --output-json /tmp/stale-script-reference-report.json`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
