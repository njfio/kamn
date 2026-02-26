# Plan: Issue #6075

## Approach
1. Drive epic closure through story/task hierarchy: `#6075 -> #6076 -> #6077`.
2. Use merged PR #6078 as the implementation anchor for the first real end-to-end delivery slice.
3. Ensure epic ACs are proven by live runtime/service integration tests and restart durability checks.
4. Complete lifecycle/traceability updates across milestone/spec artifacts and issue status markers.

## Affected Modules
- `specs/6075/*`
- `specs/6076/*`
- `specs/6077/spec.md`
- `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Runtime/test modules delivered under #6077 (`daemon_phase`, runtime tests, service API tests)

## Risks / Mitigations
- Risk: epic closure without explicit parent-child evidence chain.
  Mitigation: link epic->story->task and merged PR evidence in closure comments.
- Risk: post-merge drift in runtime tests undermines epic claims.
  Mitigation: rerun full `kamn-node` and mutation diff gates on top of current `main`.

## Interfaces / Contracts
- End-to-end delivery contract spans send ingress, daemon relay forwarding, recipient mailbox/message routes, and durable restart state.
