# Issue #4978 Spec

- Title: Subtask: enforce ratchet-only threshold update workflow and waiver-mitigation issue linkage
- Status: Implemented
- Type: subtask
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Issue #4978 needs deterministic enforcement that threshold files only ratchet downward and that any temporary waiver is explicitly time-bounded and linked to a mitigation issue.

## Acceptance Criteria
- AC-1: A threshold ratchet checker enforces non-increasing updates for `HARD_SHELL_LOC_MAX`, `WARN_SHELL_RUST_RATIO_MAX`, and `FAIL_SHELL_RUST_RATIO_MAX` against baseline values.
- AC-2: Waiver handling is fail-closed and requires a valid mitigation linkage (`#<issue-id>`), schema version, and expiry metadata.
- AC-3: `ci-fast-gate` runs the threshold-ratchet check and publishes a deterministic JSON report artifact.
- AC-4: Deterministic tests cover pass/fail/exception/wiring paths and remain green in scoped CI suites.

## Scope
In scope:
- `scripts/ci/check_shell_surface_threshold_ratchet.py`
- `scripts/ci/check_shell_surface_threshold_ratchet.sh`
- `.ci/shell-surface-threshold-ratchet-exception.json`
- `scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- CI wiring and command-surface contract updates for this checker.

Out of scope:
- Changes to shell LOC thresholds themselves.
- New governance policy outside threshold-ratchet enforcement.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Run checker against unchanged baseline | `status=pass`, `threshold_ratchet_status=within` |
| C-02 | AC-1/AC-2 | Regression | Simulate threshold increase over baseline | Fail closed with deterministic reason code and violation list |
| C-03 | AC-2 | Regression | Provide valid exception with `mitigation_issue=#<id>` | `threshold_ratchet_status=exception-applied` and pass |
| C-04 | AC-2 | Regression | Provide invalid/expired exception | Fail with `shell_surface_threshold_ratchet_exception_file_invalid` |
| C-05 | AC-3 | Integration | Run fast-gate wiring contract checks | Workflow includes checker step, exception file, and JSON artifact |
| C-06 | AC-4 | Integration/Regression | Run CI tools fast suite | Checker tests and related shell-surface contracts pass |

## Test Mapping
- AC-1:
  - `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
  - `bash scripts/ci/check_shell_surface_threshold_ratchet.sh --base-ref origin/main --hard-ceiling-file .ci/shell-loc-hard-ceiling.env --ratio-threshold-file .ci/shell-rust-ratio-guardrail.env --ratchet-exception-file .ci/shell-surface-threshold-ratchet-exception.json --output-json <tmp>`
- AC-2:
  - `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh` (exception valid/invalid coverage)
  - `python3 scripts/ci/check_shell_surface_threshold_ratchet.py --baseline-hard-ceiling-file <tmp> --baseline-ratio-threshold-file <tmp> --hard-ceiling-file .ci/shell-loc-hard-ceiling.env --ratio-threshold-file .ci/shell-rust-ratio-guardrail.env --output-json <tmp>` (negative regression path)
- AC-3:
  - `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
  - `bash scripts/ci/test_ci_tools_command_surface_contract.sh`
- AC-4:
  - `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## Success Metrics
- All ACs map to deterministic conformance cases and passing tests.
- `ci-fast-gate` now enforces threshold-ratchet behavior and emits `ci-shell-surface-threshold-ratchet.json`.
- No shell-surface contract regressions in the fast CI tools suite.
