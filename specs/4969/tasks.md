# Issue #4969 Tasks

- Issue: #4969
- Status: Implemented

## Ordered Tasks
- [x] T1 (Red): prove legacy (kolme-only) inventory fails against expanded deletion manifest.
  - Evidence: `python3 scripts/ci/superseded_script_inventory.py check ...` with temp legacy inventory -> `status=fail`, `reason_codes=superseded_deletion_manifest_references_unknown_script`, `unknown_manifest_entry_count=15`.
- [x] T2 (Green): add canary/ci/deploy/governance lanes to migration groups + ownership and regenerate baseline inventory.
  - Evidence: `inventory_entry_count=40` from generator; baseline updated deterministically.
- [x] T3 (Green): synchronize deletion manifest with expanded inventory entries.
  - Evidence: manifest deletion entry count updated to 40.
- [x] T4 (Regression): enforce non-kolme wave membership in manifest checker tests.
  - Evidence: `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh` passes with explicit 15-entry assertion.
- [x] T5 (Verify): run scoped regression suites.
  - Evidence:
    - `bash scripts/ci/test_check_superseded_script_deletion_manifest.sh`
    - `bash scripts/ci/test_check_stale_script_references.sh`
    - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
    - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## Completion Evidence
- AC-mapped conformance cases C-01..C-04 satisfied with deterministic red/green and regression checks.
