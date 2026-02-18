# Issue #4968 Tasks

- Issue: #4968
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): add failing tests derived from issue ACs and conformance cases.
- [x] T2 (Green): implement minimum change to satisfy tests deterministically.
- [x] T3 (Refactor): simplify and harden without changing behavior.
- [x] T4 (Regression): add drift/tamper/marker parity regression checks.
- [x] T5 (Docs): update required docs/process markers for issue #4968.
- [x] T6 (Verify): run scoped unit/functional/integration/regression checks and record evidence.

## Completion Evidence
- `bash scripts/ci/generate_superseded_script_inventory.sh --output-json fixtures/ci/superseded_script_inventory_baseline.json`
- `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
- `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
