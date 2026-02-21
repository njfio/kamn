# Issue #3998 Plan

## Approach

1. Close out parent task by mapping accepted ACs to delivered child implementations:
   - `#4004` local-heavy capacity/load runner contract + docs markers.
   - `#4005` capacity dry-run threshold taxonomy + docs/test parity.
2. Add parent-level spec/plan/tasks artifacts for repository traceability and milestone closure.
3. Verify parent AC coverage through targeted test command mapping in `specs/3998/spec.md`.

## Affected Modules

- `specs/3998/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations

- Risk: parent task remains open without traceable AC-to-test mapping despite completed child subtasks.
  - Mitigation: document explicit conformance and test mapping at parent level, then close with linked merged PRs.
- Risk: drift between parent objective and delivered child contracts.
  - Mitigation: map each parent AC to concrete commands and artifact markers delivered by child issues.

## Interfaces and Contracts

- No new runtime/code interfaces introduced in this closeout PR.
- Parent contract derives from merged child contracts:
  - `kamn.runtime.local-heavy-capacity-load-lane-report.v1`
  - `kamn.runtime.local-heavy-capacity-load-reason-taxonomy.v1`
  - `kamn.ci.capacity-ci-dry-run-governance-reason-taxonomy.v1`
