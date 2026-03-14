# SDK TCP Vertical Slice

This runbook documents one current KAMN SDK TCP signed-relay slice on `main` that proves a real transport-backed path with two identities, signed handshake acceptance, one successful relay, and explicit replay or tamper rejection outcomes.

## Scope
- Runtime surface: `kamn-sdk` TCP relay adapter and the TCP signed-relay examples.
- Identities: one sender DID and one listener DID.
- Success path: signed handshake acceptance and one successful TCP relay.
- Failure path: replay or tamper rejection through the real TCP adapter contract surface.

## Preconditions
- Clean checkout on current `main`.
- Rust toolchain able to build `kamn-sdk` examples and tests.
- No external services are required.

## Run
```bash
bash scripts/sdk/run_tcp_signed_relay_demo.sh
cargo test -p kamn-sdk --test tcp_transport_adapter replay_nonce_is_rejected_across_reconnect -- --nocapture
cargo test -p kamn-sdk --test tcp_transport_adapter forged_handshake_frame_is_rejected -- --nocapture
```

## Expected Evidence
- two identities participate in the relay flow
- signed handshake acceptance is evidenced by `verified=true`
- successful relay is evidenced by `status=ok`, `adapter=tcp`, and `tcp signed relay demo completed.`
- replay or tamper rejection is evidenced by:
  - `tcp handshake replay detected`
  - `handshake.signature`

## What This Proves
- The current SDK TCP path can complete one real signed relay between two identities.
- The listener verifies the signed handshake and envelope before accepting the message.
- The TCP adapter fails closed on nonce replay across reconnect.
- The TCP adapter fails closed on forged handshake signatures.

## What This Does Not Prove
- It does not prove task lifecycle, escrow settlement, bridge finality, or multi-node consensus.
- It does not prove production deployment readiness or long-lived network resilience.
- It does not prove wire compatibility with non-Rust SDK implementations.
