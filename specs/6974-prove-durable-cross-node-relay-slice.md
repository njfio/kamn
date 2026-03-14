# 6974-prove-durable-cross-node-relay-slice

## Objective
Prove one current, operator-comprehensible KAMN durable cross-node relay slice on `main` that exercises the real service-API sender spool, daemon relay projection, fail-closed retry behavior, and recipient-visible durable delivery state across restart or fresh-reader boot.

## Inputs/Outputs
- Inputs:
  - current service API delivery runtime and daemon relay projection
  - existing integration coverage for relay spool enqueue, cross-node relay delivery, and route-map failure behavior
  - existing delivery-flow documentation under `docs/architecture/service-api-delivery-flow.md`
- Outputs:
  - one dedicated operator-facing validation doc under `docs/validation/`
  - one hard-fail regression contract tying the doc to the exact executable proof path
  - any minimal doc/test wiring needed to make this slice explicit and reproducible on current `main`

## Boundaries/Non-goals
- Do not redesign service API routes or relay mechanics.
- Do not add dependencies.
- Do not introduce new transport layers or consensus claims.
- Do not claim escrow settlement, bridge finality, or production deployment readiness.
- Do not add mock-only runtime adapters.

## Failure modes
- The proof documents only successful delivery and omits fail-closed pending-spool behavior.
- The proof claims restart durability without pointing to a real path that reads the same durable state after restart or fresh boot.
- The regression contract can pass while the validation doc drifts away from the executable tests.
- The proof overclaims beyond sender spool durability, relay retry, recipient delivery visibility, and durable state continuity.
- The validation doc is left floating and not clearly tied to the runtime delivery flow already on `main`.

## Acceptance criteria
- [ ] A dedicated durable cross-node relay validation doc exists on current `main`.
- [ ] The doc explicitly covers all of the following in one coherent slice:
  - sender enqueue to relay spool
  - fail-closed pending spool preservation when routing or recipient availability is missing
  - later successful relay projection to a live recipient
  - recipient-visible delivered state from the durable state file after restart or fresh-reader boot
- [ ] A hard-fail regression contract ties the doc to the exact proof markers and executable path.
- [ ] The proof uses current-main integration coverage rather than synthetic-only markers.
- [ ] The spec records exactly what this relay slice proves and what remains out of scope.

## Files to touch
- `specs/6974-prove-durable-cross-node-relay-slice.md`
- one new proof doc under `docs/validation/`
- one new regression contract under `crates/kamn-node/tests/`
- only minimal doc/test/runtime files required to keep the proof honest

## Error semantics
- Missing spool-preservation markers, missing relay-success markers, or missing durable-reader markers must fail loudly in the regression contract.
- The proof must not silently rely on in-memory state while claiming durable restart continuity.
- Any documented failure behavior must remain explicit and operator-readable.

## Test plan
- Phase 3 red test that fails because the dedicated proof doc/contract does not yet exist.
- Add one hard-fail regression contract asserting required proof markers in the validation doc and linked runtime/test path.
- Re-run the current relay spool enqueue and cross-node relay delivery integration coverage.
- Re-run or add minimal current-main coverage proving recipient-visible delivery from the same durable state after restart or fresh boot.
- Final verification should include the new proof contract, the referenced integration target(s), and touched-Rust policy.

## Execution notes
This issue exists to deepen the runtime proof surface beyond happy-path vertical slices. If the current service-API delivery path cannot honestly prove durable retry and restart semantics, the issue must record the exact blocker instead of inflating claims.

## Final implementation
- Validation runbook: [docs/validation/durable-cross-node-relay-slice.md](/home/n/Code/kamn/docs/validation/durable-cross-node-relay-slice.md)
- Hard-fail proof contract: [durable_cross_node_relay_slice_contract.rs](/home/n/Code/kamn/crates/kamn-node/tests/durable_cross_node_relay_slice_contract.rs)
- Delivery-flow wiring: [service-api-delivery-flow.md](/home/n/Code/kamn/docs/architecture/service-api-delivery-flow.md)

## Final evidence
- `cargo test -p kamn-node --test durable_cross_node_relay_slice_contract -- --nocapture`
- `cargo test -p kamn-node integration_runtime_daemon_without_route_map_preserves_relay_spool_entries -- --exact --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_cross_node_relay_delivery_contract -- --exact --nocapture`
- `cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --exact --nocapture`
- `cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --exact --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6974-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Observed proof
- Pending relay spool entries are preserved fail-closed when routing is unavailable.
- A later daemon relay projection drains the sender relay spool and advances sender state.
- The recipient side exposes durable `delivered` state from the persisted state file after a fresh reader boot.
- Non-recipient restart reads keep `relayed` stable instead of incorrectly promoting delivery.

## Deviations
- None. Current `main` already contained the required runtime coverage; this issue formalized it as one operator-readable proof.
