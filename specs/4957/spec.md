# Issue #4957 Spec

- Title: Story: enforce hard shell LOC ceiling and downward-only shell-to-Rust ratio ratchet in CI
- Status: Implemented
- Type: story
- Priority: P0
- Milestone: specs/milestones/r27-44-shell-loc-deletion-wave-and-hard-ceiling-governance/index.md

## Problem Statement
Sustaining shell-surface improvements required hard merge-blocking gates for shell LOC ceiling, ratio guardrails, and ratchet/waiver governance.

## Acceptance Criteria
- AC-1: CI fails when shell LOC exceeds configured hard ceiling.
- AC-2: CI fails when shell-rust ratio trajectory regresses beyond ratchet policy.
- AC-3: Ratchet waivers require deterministic mitigation metadata linkage.
- AC-4: Policy outputs and CI wiring remain deterministic and tested.

## Scope
In scope:
- Hard-ceiling checker lifecycle closure.
- Fast-gate integration for ceiling + ratio checks.
- Ratchet/waiver governance checker and workflow enforcement.

Out of scope:
- Non-shell budget governance.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh` | ceiling violations fail deterministically |
| C-02 | AC-2 | Regression | `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh` | ratio regression gate works |
| C-03 | AC-3 | Functional/Regression | `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh` | waiver linkage and ratchet rules enforced |
| C-04 | AC-4 | Integration | `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh` | CI required-check wiring remains valid |

## Test Mapping
- AC-1: `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh`
- AC-2: `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
- AC-3: `bash scripts/ci/test_check_shell_surface_threshold_ratchet.sh`
- AC-4: `bash scripts/ci/test_fast_gate_shell_surface_ratio_policy_wiring.sh`

## Success Metrics
- CI merge-blocking shell governance story fully enforced with deterministic output contracts.
