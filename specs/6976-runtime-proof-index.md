# 6976-runtime-proof-index

## Objective
Publish one operator-facing index on current `main` that summarizes the currently proven KAMN runtime slices, links each proof runbook, and states the boundaries of those proofs without inflating product maturity claims.

## Inputs/Outputs
- Inputs:
  - current validation runbooks under `docs/validation/`
  - current corrected audit response under `docs/review/`
  - current executable proof contracts already on `main`
- Outputs:
  - one validation index under `docs/validation/`
  - one hard-fail docs contract ensuring the index keeps the three proof links and key summary markers
  - minimal wiring from an existing validation/docs surface so the index is discoverable

## Boundaries/Non-goals
- Do not change runtime behavior.
- Do not add dependencies.
- Do not claim production readiness, consensus maturity, or settlement guarantees.
- Do not duplicate full runbook content; summarize and link.

## Failure modes
- The index omits one of the three current proof slices.
- The index restates claims more broadly than the underlying proof docs justify.
- The docs contract can pass while the index drifts away from the current proof set.
- The index is left floating without any discoverability wiring.

## Acceptance criteria
- [ ] A dedicated runtime-proof index exists under `docs/validation/`.
- [ ] The index links all three current proof slices.
- [ ] The index states what each slice proves in bounded language.
- [ ] The index states what remains unproven overall.
- [ ] A hard-fail docs contract enforces the three links and key summary markers.
- [ ] The finished index is reachable from an existing validation/docs surface on current `main`.

## Files to touch
- `specs/6976-runtime-proof-index.md`
- one new index doc under `docs/validation/`
- one new docs contract under `crates/kamn-node/tests/` or another existing validation-contract surface
- one existing validation/docs surface for discoverability wiring

## Error semantics
- Missing proof links, missing scope markers, or missing unproven-boundary markers must fail loudly.
- The index must not silently expand the proof scope beyond what the linked runbooks already demonstrate.

## Test plan
- Phase 3 red test that fails because the index doc/contract does not yet exist.
- One hard-fail docs contract asserting:
  - the three proof-runbook links remain present
  - the index states current proven slices
  - the index states remaining unproven areas
- Re-run the new docs contract and touched-Rust policy.

## Execution notes
This issue exists to give operators and reviewers one current-main entrypoint for runtime substance. It should reduce repetitive repo-wide argument and point directly at the three proofs already established on `main`.
