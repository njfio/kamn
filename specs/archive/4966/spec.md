# Issue #4966 Spec

- Title: Task: enforce downward-only shell-budget ratchet updates and waiver governance workflow
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The milestone required explicit ratchet-only governance and waiver linkage rules so threshold regressions cannot be merged without deterministic mitigation metadata.

## Acceptance Criteria
- AC-1: Threshold updates are ratchet-only by default.
- AC-2: Regression waivers require linked mitigation issue metadata.
- AC-3: Checker emits deterministic reason taxonomy and JSON telemetry.
- AC-4: Ratchet checker and CI wiring contract tests pass.

## Scope
In scope:
- Shell-surface threshold-ratchet checker, workflow wiring, and waiver metadata enforcement.

Out of scope:
- Non-shell budget governance domains.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | ratchet threshold update validation | non-ratchet changes fail |
| C-02 | AC-2 | Regression | waiver without mitigation issue | deterministic NO-GO |
| C-03 | AC-3 | Unit | checker JSON/reason output parsing | deterministic schema/taxonomy fields |
| C-04 | AC-4 | Integration | fast-gate + checker contract suites | ratchet governance lane passes |

## Test Mapping
- AC-1/AC-2/AC-3: `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- AC-4: `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`, `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## Success Metrics
- Ratchet/waiver governance is fail-closed and merge-blocking in CI.
