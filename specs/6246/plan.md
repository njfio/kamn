# Issue 6246 Plan

## Approach
1. Re-check each audited top-10 item against current code and CI configuration.
2. Classify each item as complete or requiring follow-up.
3. Create a single coordinating story and one child task per unresolved area.
4. Create per-task spec artifacts so implementation can start under spec-first gates.
5. Record baseline metrics (coverage thresholds, E2E PR job behavior, shell/rust ratio) in a follow-up planning document.

## Affected Modules
- `specs/6246/{spec.md,plan.md,tasks.md}`
- `specs/6247/{spec.md,plan.md,tasks.md}`
- `specs/6248/{spec.md,plan.md,tasks.md}`
- `specs/6249/{spec.md,plan.md,tasks.md}`
- `specs/6250/{spec.md,plan.md,tasks.md}`
- `docs/planning/r59-followup.md`
- `specs/milestones/r59-swarm-gap-closure/index.md`

## Risks and Mitigations
- Risk: follow-up tasks duplicate already-complete work.
  - Mitigation: use direct repository evidence and current CI behavior to justify each new task.
- Risk: task boundaries overlap and create churn.
  - Mitigation: isolate tasks by measurable outcome (coverage, PR E2E, extraction, shell ratio).
- Risk: implementation starts without accepted specs.
  - Mitigation: create and mark per-task artifacts as `Reviewed` before coding begins.

## Interfaces
- No runtime API or wire-format changes.
- Issue tracking and spec-document interfaces only.
