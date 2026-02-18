# Issue #4965 Spec

- Title: Task: wire shell ceiling and ratio-ratchet checks into CI fast gate as merge blockers
- Status: Implemented
- Type: task
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
The milestone required fast-gate CI wiring so shell hard-ceiling and shell-rust ratio checks execute as merge-blocking gates.

## Acceptance Criteria
- AC-1: Fast-gate workflow runs shell ceiling and ratio checks on PRs.
- AC-2: Violations produce failing required check status.
- AC-3: Wiring remains compatible with CI runtime/cost constraints.
- AC-4: Fast-gate policy wiring contracts pass.

## Scope
In scope:
- CI fast-gate integration evidence and lifecycle finalization for required checks.

Out of scope:
- Ratchet waiver governance rules themselves (covered by #4966).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | fast-gate run on PR | ceiling/ratio checks execute |
| C-02 | AC-2 | Regression | induced policy violation fixtures | required check fails |
| C-03 | AC-3 | Performance | fast-mode CI contract run | bounded runtime/cost behavior |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` | wiring suite passes |

## Test Mapping
- AC-1/AC-2/AC-4: `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`
- AC-3: `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh`

## Success Metrics
- Fast-gate required status checks include shell ceiling + ratio gates and remain deterministic.
