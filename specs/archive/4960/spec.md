# Issue #4960 Spec

- Title: Task: add stale-script reference detector and fail-closed CI guard for deleted entrypoints
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4960 is part of the R27.44 shell maintainability tranche and closes one portion of the deletion-wave, spec-archival, and hard-ceiling governance gap.

## Acceptance Criteria
- AC-1: Scope defined in GitHub issue #4960 is implemented and verified.
- AC-2: Deterministic fail-closed behavior is preserved for drift/regression scenarios.
- AC-3: Required Unit/Functional/Integration/Regression tests are present and passing.
- AC-4: Documentation/process markers remain synchronized where issue scope requires docs updates.

## Scope
In scope:
- Work explicitly described in issue #4960.

Out of scope:
- Unrelated feature expansion outside the issue boundary.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Execute scoped workflow for #4960 | Behavior matches issue acceptance criteria |
| C-02 | AC-2 | Regression | Inject marker/schema/taxonomy drift scenario | Policy/output fails closed with deterministic reasons |
| C-03 | AC-3 | Unit/Integration | Run scoped tests and lane checks | Required suites pass |
| C-04 | AC-4 | Functional/Regression | Validate docs/process marker contract checks | Marker parity remains verified |

## Test Mapping
- AC-1:
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- AC-2:
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/check_stale_script_references.sh --output-json /tmp/stale-script-reference-report.json`
- AC-3:
  - `bash scripts/ci/test_check_stale_script_references.sh`
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `cargo test -p kamn-core checklist_contains_stale_script_reference_deletion_wave_gate -- --exact`

## Success Metrics
- All ACs for #4960 are mapped to conformance cases and passing tests.
- No shell-surface governance regressions introduced by #4960.
