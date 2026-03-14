# 6972-corrected-audit-response

## Objective
Publish one corrected audit response on current `main` that evaluates the recent external KAMN critique against the code as it exists today, explicitly anchored to the two proof slices now present on `main`: the node service-API vertical slice and the SDK TCP signed-relay vertical slice.

## Inputs/Outputs
- Inputs:
  - current review critique text and the claims already discussed in issue `#6972`
  - current proof docs under `docs/validation/`
  - current executable contracts proving those slices
  - current repo telemetry available from the working tree
- Outputs:
  - one review response doc under `docs/review/`
  - one hard-fail docs contract tying that review doc to the current proof anchors and corrected conclusions
  - minimal review index wiring so the response is discoverable from the existing review surface

## Boundaries/Non-goals
- Do not rewrite historical `gaps-and-issues-r*.md` review artifacts.
- Do not claim full product readiness, production deployment, or consensus maturity.
- Do not invent telemetry that cannot be reproduced from the current checkout.
- Do not add dependencies.
- Do not expand this into new runtime implementation work beyond any minimal doc/test wiring needed to keep the response honest.

## Failure modes
- The review response repeats stale or inflated numbers without correcting them.
- The review response omits the two current proof anchors and therefore overstates hollowness.
- The review response overcorrects and claims maturity that current `main` does not prove.
- The docs contract can pass while the review doc drifts away from the actual proof paths or corrected conclusions.
- The review doc is left unwired from the review index and becomes another floating artifact.

## Acceptance criteria
- [ ] A dedicated corrected audit response doc exists under `docs/review/` on current `main`.
- [ ] The doc explicitly references both proof anchors:
  - `docs/validation/working-vertical-slice.md`
  - `docs/validation/sdk-tcp-vertical-slice.md`
- [ ] The doc separates at least three classes of claims: accurate, stale/incorrect, and unproven.
- [ ] The doc records corrected current-main telemetry where available instead of repeating stale audit numbers.
- [ ] A hard-fail regression contract enforces the presence of the proof anchors and key corrected conclusions.
- [ ] The finished review doc is linked from the existing `docs/review/` surface.

## Files to touch
- `specs/6972-corrected-audit-response.md`
- one new review doc under `docs/review/`
- one new docs contract under `crates/kamn-core/tests/` or another existing docs-contract surface
- `docs/review/README.md` for discoverability wiring
- only minimal files required to keep the review honest and enforced

## Error semantics
- Missing proof-anchor references, missing corrected-telemetry markers, or missing classification of claims must fail loudly in the docs contract.
- The review response must not silently downgrade proven runtime paths into hypothetical claims.
- The review response must not silently upgrade unproven areas into working guarantees.

## Test plan
- Phase 3 red test that fails because the corrected review doc/contract does not yet exist.
- Add one hard-fail docs contract asserting:
  - both proof-doc paths are referenced
  - build-health blockers cited by the stale audit are marked fixed
  - the stale `~23K Rust LOC` and `8,536 tests` claims are explicitly corrected
  - AGENTS size debt remains acknowledged
- Re-run the new docs contract and any directly related docs/index contract coverage needed to keep the review surface honest.
- Final verification should include the new docs contract and the touched-Rust policy gate.

## Execution notes
This issue is not a marketing rebuttal. It exists to make the repo's current state defensible: what the critique gets right, what is now stale, what remains unproven, and what two concrete runtime proofs now exist on `main`.
