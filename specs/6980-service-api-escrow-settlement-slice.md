# 6980-service-api-escrow-settlement-slice

## Objective
Prove one current, operator-comprehensible KAMN service-API escrow settlement slice on `main` that states exactly which escrow lifecycle behavior is executable today through the service API, using existing route and restart integration tests rather than broader settlement or chain-finality claims.

## Inputs/Outputs
- Inputs:
  - current service-api task and escrow route tests under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
  - current restart-persistence proof and runtime proof index docs on `main`
  - current service-api audit export behavior for task and escrow lifecycle operations
- Outputs:
  - one dedicated escrow-settlement validation doc under `docs/validation/`
  - one hard-fail docs contract binding that doc to the exact executable escrow proof anchors
  - minimal docs wiring so the proof is discoverable from the current proven runtime slices index

## Boundaries/Non-goals
- Do not claim external chain finality.
- Do not claim Byzantine-safe or adversarial escrow settlement.
- Do not redesign task or escrow persistence.
- Do not add dependencies.
- Do not expand this into bridge finality or multi-node consensus claims.

## Failure modes
- The proof claims escrow settlement guarantees stronger than the current service-api tests actually provide.
- The proof omits either the route-level fund/release evidence or the restart-persistence evidence for released escrow state.
- The docs contract can pass while the validation doc drifts away from the executable escrow anchors.
- The runtime proof index continues to list escrow settlement as unproven after this bounded proof is added.

## Acceptance criteria (testable booleans)
- [ ] A dedicated escrow-settlement proof doc exists under `docs/validation/`.
- [ ] The proof links current executable coverage for task create/accept, escrow fund, and escrow release on the real service-api path.
- [ ] The proof links current executable coverage for released escrow state remaining persisted across restart.
- [ ] The proof states clearly that it proves service-api escrow lifecycle persistence, not bridge finality, external chain settlement, or adversarial settlement.
- [ ] A hard-fail docs contract enforces the required proof anchors and bounded language.
- [ ] The finished proof is wired into `docs/validation/current-proven-runtime-slices.md`.

## Files to touch
- `specs/6980-service-api-escrow-settlement-slice.md`
- one new validation doc under `docs/validation/`
- one new docs contract under `crates/kamn-node/tests/`
- `docs/validation/current-proven-runtime-slices.md`

## Error semantics
- Missing proof anchors or missing boundary markers must fail loudly.
- The proof must not silently equate service-api escrow lifecycle persistence with chain-backed settlement finality.

## Test plan
- Phase 3 red test that fails because the escrow-settlement validation doc and contract do not yet exist.
- One hard-fail docs contract asserting required escrow proof links and bounded-language markers.
- Re-run the referenced service-api task/escrow route and restart targets and touched-Rust policy before publish.
