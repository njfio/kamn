# Plan - Issue #3647

## Approach

1. Validate merge-gate evaluator and anti-flake policy checks.
2. Validate deterministic evidence report and registry-sync behavior.
3. Close task with AC-mapped lifecycle artifacts.

## Affected Paths

- `specs/3647/spec.md`
- `specs/3647/plan.md`
- `specs/3647/tasks.md`

## Risks / Mitigations

- Risk: unresolved flaky conditions may leak through if merge policy drifts.
  Mitigation: require deterministic gate-policy and evidence-report suites in closure verification.

## ADR

- Not required (lifecycle artifact closure only).
