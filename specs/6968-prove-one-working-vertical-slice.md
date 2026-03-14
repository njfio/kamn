# 6968-prove-one-working-vertical-slice

## Objective
Prove one current, operator-comprehensible KAMN vertical slice on `main` that exercises real runtime paths end-to-end: two identities, encrypted message delivery, at least one task lifecycle transition, persisted/auditable evidence, and one reproducible integration or e2e test that fails closed on regressions.

## Inputs/Outputs
- Inputs:
  - current `main` runtime surfaces in `kamn-core`, `kamn-node`, `kamn-sdk`, and `kamn-e2e-harness`
  - existing live/local integration and e2e lanes
  - current docs/spec infrastructure
- Outputs:
  - one chosen vertical-slice runtime path documented and executable from a clean checkout
  - one integration/e2e contract covering that exact path
  - one operator-facing doc/runbook section explaining setup, execution, evidence, and current limits
  - evidence artifacts or logs that show delivery, task transition, and audit/persistence outputs

## Boundaries/Non-goals
- Do not redesign the protocol or crate layout in this issue.
- Do not broaden this into a general “production readiness” program.
- Do not claim bridge finality, Byzantine settlement, or multi-node fault tolerance unless the slice actually proves them.
- Do not delete CI/policy infrastructure here.
- Do not add speculative demo-only code that bypasses real runtime entrypoints.

## Candidate vertical slice
Preferred slice unless implementation evidence shows a better current path:
1. bootstrap local runtime,
2. provision two real KAMN identities,
3. send one encrypted message from agent A to agent B through the real service/runtime path,
4. create one task tied to that interaction,
5. advance the task through at least one real dispatch/assignment/completion transition,
6. verify persisted evidence and audit/export artifacts,
7. produce one operator-readable summary of what happened.

## Failure modes
- The chosen flow still depends on mock-only adapters or synthetic shortcuts.
- Message delivery succeeds only as storage/retrieval and not as a real end-to-end slice.
- Task lifecycle evidence is not tied to the same demonstrated flow.
- Audit/persistence outputs are missing, unstable, or not operator-verifiable.
- The demo requires undocumented setup or hidden local state.
- The added integration/e2e test passes without exercising the same runtime path described in the doc.

## Acceptance criteria
- [ ] A single documented vertical slice is identified and implemented against current `main`.
- [ ] The slice is runnable from a clean checkout with explicit commands and prerequisites.
- [ ] The slice uses real application entrypoints and real wiring, not mock-only test seams.
- [ ] The slice proves two identities, encrypted delivery, one task transition, and persisted/auditable evidence in one coherent flow.
- [ ] At least one integration or e2e test covers the exact slice and fails closed on regressions.
- [ ] Operator-facing documentation explains what the demo proves, what evidence to inspect, and what is still out of scope.

## Files to touch
- likely docs/spec/runbook paths under `docs/` and `specs/`
- one or more existing integration/e2e test paths in `crates/kamn-node/tests/`, `crates/kamn-sdk/tests/`, or `crates/kamn-e2e-harness/tests/`
- only the minimal runtime files required to wire or repair the chosen slice

## Error semantics
- Preserve hard-fail behavior throughout the demonstrated path.
- Missing config, missing evidence artifacts, invalid task transitions, and failed delivery must surface as explicit errors.
- The demo documentation must identify expected failure points and how they present.

## Test plan
- Phase 3 red test that asserts the chosen slice is not yet sufficiently proven on current `main`, or that the target doc/test artifact is missing.
- One integration or e2e contract for the chosen slice.
- Any targeted runtime tests needed to keep the real path green.
- Final verification should include:
  - the new slice contract test,
  - the directly related runtime/integration targets,
  - touched-Rust policy if Rust changes are required.

## Execution notes
This issue exists to force a product proof point. If the preferred slice cannot be wired honestly on current `main`, the issue must record the exact blocker and either narrow the slice or open concrete follow-on issues for the missing runtime capability.
