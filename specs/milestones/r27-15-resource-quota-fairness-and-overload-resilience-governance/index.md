# R27.15 Resource Quota Fairness and Overload Resilience Governance

## Milestone Summary

Closure tranche focused on deterministic overload governance:
- per-scope quota policy parsing/checking,
- starvation/fairness guardrails,
- CI-smoke policy checks with explicit local-heavy boundaries for deep stress lanes.

Milestone objective: enforce fail-closed overload behavior without regressing CI cost controls.

## Issue Hierarchy

- Epic:
  - `#4083` — Epic: R27.15 close resource quota fairness and overload resilience governance gaps
- Stories:
  - `#4084` — Story: enforce deterministic quota and fairness governance across overload-prone service scopes
  - `#4085` — Story: validate overload degradation-recovery evidence with local-heavy stress and ci dry-run gates
- Tasks (Story `#4084`):
  - `#4086` — Task: implement per-scope quota policy checker with deterministic fail-closed taxonomy
  - `#4087` — Task: implement fairness-starvation checker and docs parity governance for scheduler outcomes
- Subtasks (Task `#4086`):
  - `#4090` — Subtask: define quota policy fixture matrix and parser helper contracts
  - `#4091` — Subtask: add fail-closed quota checker and deterministic violation taxonomy tests

## Governance Markers

- `quota_policy_reason_taxonomy=fail_closed`
- `fairness_starvation_detection=required`
- `ci_smoke_overload_checker_budget=bounded`
- `local_heavy_overload_stress=opt_in_only`
