# 6970-prove-sdk-tcp-vertical-slice

## Objective
Prove one current, operator-comprehensible KAMN SDK TCP signed-relay vertical slice on `main` that exercises the real TCP relay adapter path end-to-end: two identities, signed handshake acceptance, one successful relay, one explicit replay/tamper rejection signal, and one reproducible operator command that fails closed on regressions.

## Inputs/Outputs
- Inputs:
  - current `kamn-sdk` TCP relay adapter and handshake implementation
  - existing TCP demo runner `scripts/sdk/run_tcp_signed_relay_demo.sh`
  - existing transport tests under `crates/kamn-sdk/tests/`
  - current SDK docs under `docs/foundation/rust-sdk-alpha.md`
- Outputs:
  - one dedicated operator-facing proof doc for the SDK TCP signed-relay slice
  - one hard-fail regression contract tying that doc to the exact executable proof path
  - any minimal runtime/doc/test wiring needed to make the proof honest and reproducible on current `main`

## Boundaries/Non-goals
- Do not redesign the TCP transport or handshake protocol.
- Do not add dependencies.
- Do not expand this into service-api, bridge, or consensus proof work.
- Do not claim task lifecycle, escrow settlement, or production deployment readiness from this transport slice.
- Do not add mock-only demo adapters.

## Failure modes
- The proof doc points at a script or test path that no longer reflects the real TCP adapter behavior.
- The slice demonstrates only a happy-path relay and omits replay/tamper failure evidence.
- The proof relies on synthetic markers not emitted by the actual demo/runtime path.
- The regression contract can pass while the operator doc drifts from the executable path.
- The proof overclaims scope beyond signed handshake, relay, and explicit rejection signals.

## Acceptance criteria
- [x] A dedicated SDK TCP vertical-slice proof doc exists on current `main`.
- [x] The doc is runnable from a clean checkout with explicit commands and prerequisites.
- [x] The doc demonstrates two identities, signed handshake acceptance, one successful relay, and at least one replay or tamper rejection outcome.
- [x] At least one hard-fail regression contract ties the doc to the exact executable proof path.
- [x] The spec records exactly what this transport proof demonstrates and what remains out of scope.

## Files to touch
- `specs/6970-prove-sdk-tcp-vertical-slice.md`
- likely one new proof doc under `docs/validation/`
- likely one new regression contract under `crates/kamn-sdk/tests/`
- only minimal doc/test/runtime files required to keep the proof honest

## Error semantics
- Missing doc artifacts, missing proof commands, missing replay/tamper rejection markers, or drift between the doc and executable path must fail loudly.
- The proof must not silently fall back to mock/in-memory transport.
- Any documented failure signal must remain explicit and operator-readable.

## Test plan
- Phase 3 red test that fails because the dedicated proof doc/contract does not yet exist.
- One hard-fail regression contract asserting required markers in the proof doc and executable path.
- Re-run the existing TCP signed-relay demo validation path and any directly related SDK TCP tests needed to keep the path green.
- Final verification should include:
  - the new proof contract
  - the existing TCP demo script test
  - one direct SDK TCP transport target covering successful relay and rejection behavior

## Execution notes
This issue exists to make one current transport-backed slice defensible when evaluating whether KAMN has real runtime substance. If the existing TCP relay path cannot be documented honestly with explicit failure evidence, the issue must record the exact blocker instead of inflating claims.


## Final implementation
- Dedicated proof doc: [docs/validation/sdk-tcp-vertical-slice.md](/home/n/Code/kamn/docs/validation/sdk-tcp-vertical-slice.md)
- Hard-fail regression contract: [sdk_tcp_vertical_slice_contract.rs](/home/n/Code/kamn/crates/kamn-sdk/tests/sdk_tcp_vertical_slice_contract.rs)
- Existing SDK docs now link the proof from [rust-sdk-alpha.md](/home/n/Code/kamn/docs/foundation/rust-sdk-alpha.md)

## Final evidence
- `cargo test -p kamn-sdk --test sdk_tcp_vertical_slice_contract -- --nocapture`
- `bash scripts/sdk/test_run_tcp_signed_relay_demo.sh`
- `cargo test -p kamn-sdk --test tcp_transport_adapter replay_nonce_is_rejected_across_reconnect -- --nocapture`
- `cargo test -p kamn-sdk --test tcp_transport_adapter forged_handshake_frame_is_rejected -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6970-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Observed proof
- The current SDK TCP path proves one successful signed relay between a sender DID and listener DID.
- The listener emits `verified=true` and `status=ok` markers through the real demo script path.
- Replay across reconnect is rejected with `tcp handshake replay detected`.
- Forged handshake frames are rejected with `handshake.signature` classification.

## Deviations
- None. The current TCP transport surface was already sufficient; this issue formalized it as one operator-readable proof.
