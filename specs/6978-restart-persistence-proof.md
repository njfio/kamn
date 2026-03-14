# 6978-restart-persistence-proof

## Objective
Prove one current, operator-comprehensible KAMN service-API restart persistence slice on `main` that summarizes which persisted service-API state survives restart or fresh boot today, using existing executable restart contracts rather than broader crash-recovery claims.

## Inputs/Outputs
- Inputs:
  - current restart-persistence integration tests under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
  - current durable relay/restart validation docs already on `main`
  - current architecture and audit-response docs that need bounded restart language
- Outputs:
  - one dedicated restart-persistence validation doc under `docs/validation/`
  - one hard-fail docs contract tying that doc to the exact restart-persistence proof anchors
  - minimal docs wiring so the proof is discoverable from the validation surface

## Boundaries/Non-goals
- Do not claim SIGKILL/WAL crash recovery if the current proof anchors only cover restart persistence.
- Do not redesign state persistence.
- Do not add dependencies.
- Do not add mock-only runtime paths.
- Do not expand this into consensus, escrow settlement, or bridge finality claims beyond restart persistence.

## Failure modes
- The proof claims crash recovery when the underlying tests only prove restart persistence.
- The proof omits one of the major restart-persistence surfaces already present on `main`.
- The docs contract can pass while the validation doc drifts away from the executable restart anchors.
- The proof overstates breadth beyond message, task/escrow, directory, and relay-status continuity.

## Acceptance criteria
- [ ] A dedicated restart-persistence proof doc exists under `docs/validation/`.
- [ ] The proof links real current-main restart coverage for at least:
  - message state across restart
  - task and escrow state across restart
  - directory state across restart
  - relayed/delivered status continuity across restart or fresh boot
- [ ] The proof states clearly what it proves and what remains unproven.
- [ ] A hard-fail docs contract enforces the required proof anchors and bounded language.
- [ ] The finished proof is wired into an existing docs/validation or related operator-facing surface.

## Files to touch
- `specs/6978-restart-persistence-proof.md`
- one new validation doc under `docs/validation/`
- one new docs contract under `crates/kamn-node/tests/`
- one existing validation/docs surface for discoverability wiring

## Error semantics
- Missing restart-proof anchors or missing boundary markers must fail loudly.
- The proof must not silently equate restart persistence with stronger crash-recovery guarantees.

## Test plan
- Phase 3 red test that fails because the restart-persistence validation doc/contract does not yet exist.
- One hard-fail docs contract asserting required restart-proof links and bounded-language markers.
- Re-run the referenced restart-persistence integration targets and touched-Rust policy before publish.

## Execution notes
This issue exists to make current restart persistence claims defensible. If current `main` cannot support a coherent restart-persistence proof, the issue must record the exact gap rather than overstating recovery maturity.
