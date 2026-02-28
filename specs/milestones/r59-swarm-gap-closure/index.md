# R59 Swarm Gap Closure

- Milestone ID: `r59-swarm-gap-closure`
- Source review: `docs/review/gaps-and-issues-r59-swarm.md`
- Status: Active
- Scope: close all R59 critical (`C-*`), important (`I-*`), and suggestion (`S-*`) findings with issue-driven, spec-driven, test-driven implementation waves.

## Objectives

1. Close all security-blocking R59 critical gaps (C-01..C-07).
2. Close all R59 important product-hardening gaps (I-01..I-12).
3. Track and execute the R59 suggestion backlog (S-01..S-15) with explicit priority and risk.
4. Keep issue hierarchy explicit: one epic parent and one parent link per task.

## Backlog Taxonomy

- Critical (`P0`): C-01..C-07
- Important (`P1`): I-01..I-12
- Suggestions (`P2`): S-01..S-15

## Follow-Up Wave

- Coordinating story: #6246 (`Story: Reconcile and complete remaining Top-10 remediation outcomes`)
- Child tasks:
  - #6247 (critical-path coverage threshold ratchet)
  - #6248 (full PR E2E smoke coverage)
  - #6249 (wave-2 `kamn-core` extraction and shim retirement)
  - #6250 (shell/workflow/template ratio reduction and non-regression gate)
- Baseline metrics and reconciliation: `docs/planning/r59-followup.md`

## Contract Notes

1. Every implementation task must have:
   - milestone assignment,
   - required labels (`type:*`, `area:*`, `process:*`, `priority:*`, `status:*`),
   - parent link to the R59 epic issue.
2. Shell-surface tasks must include DoR shell/rust ratio estimates in the issue body.
3. Implementation starts only after per-issue `spec.md` reaches acceptance status per `AGENTS.md`.
