# Corrected Audit Response (2026-03-14)

## Scope
This response evaluates one recent external KAMN audit against current `main` as of 2026-03-14. It is not a release review and it does not rewrite the historical `gaps-and-issues-r*.md` artifacts.

The current proof entrypoint on `main` is:
- `docs/validation/current-proven-runtime-slices.md`

The current proof anchors on `main` include:
- `docs/validation/working-vertical-slice.md`
- `docs/validation/sdk-tcp-vertical-slice.md`
- `docs/validation/durable-cross-node-relay-slice.md`
- `docs/validation/restart-persistence-slice.md`
- `docs/validation/escrow-settlement-slice.md`

Those proofs matter because they establish that current `main` is not only type definitions and wrappers. The node path proves one real service-API vertical slice. The SDK path proves one real TCP signed-relay vertical slice. The newer relay, restart, and escrow proofs make the bounded persistence and settlement claims easier to inspect from one place.

## Accurate Claims
- `kamn-core` is still too large and too central.
- AGENTS size debt remains real.
- The repo still contains more architecture, policy, and documentation surface than one would want without concrete runtime proofs.
- The strongest technical surfaces remain the cryptographic discipline, typed contracts, and fail-closed validation paths.
- The strategic criticism is fair: KAMN needs more undeniable end-to-end runtime proof, not only more decomposition.

## Stale Or Incorrect Claims
- The earlier build-health blockers from the earlier audit are fixed on current main.
- The earlier audit's `~23K LOC Rust` claim is stale.
- The earlier audit's `8,536 tests` claim is stale.
- The earlier audit's exact docs/spec counts are stale.

Current reproducible telemetry from this checkout:
- Rust LOC under crates/: 93370
  - command: `rg --files crates -g '*.rs' | xargs wc -l | tail -n 1`
- direct #[test] count under crates/: 5058
  - command: `rg -n '#\[test\]' crates | wc -l`
- docs file count: 271
  - command: `find docs -type f | wc -l`
- spec file count: 3429
  - command: `find specs -type f | wc -l`

Current build-health status for the specific blockers that audit called out:
- `cargo check` passes on current `main`.
- `cargo test --no-run` no longer fails on the former M7 parse error.
- the two production `unwrap()` violations previously cited in M1 were fixed in `#6963`.

## Unproven Claims
- The repo does not yet prove broad production readiness.
- The repo does not yet prove full consensus maturity, bridge finality, or live economic settlement from one operator-facing path.
- The repo does not yet prove that every major noun in the architecture has equivalent runtime depth.
- Claims like `no real persistence` or `only scaffolding` are too absolute for current `main`, but broad product maturity is still not proven either.

## Current Proof Anchors On Main
### Node Service-API Vertical Slice
`docs/validation/working-vertical-slice.md` proves one current node-backed path with:
- two identities in one coherent runtime path
- real service-API message send plus daemon relay projection
- persisted `data_layer_runtime_evidence`
- one real task lifecycle transition to `completed`
- audit export containing `service_api_task_created`

### SDK TCP Signed-Relay Vertical Slice
`docs/validation/sdk-tcp-vertical-slice.md` proves one current SDK path with:
- one transport-backed relay between two identities
- signed handshake acceptance through the real TCP path
- one successful relay with `status=ok`, `adapter=tcp`, and `verified=true`
- explicit replay rejection
- explicit forged-handshake rejection

### Durable Relay, Restart, And Escrow Slices
`docs/validation/durable-cross-node-relay-slice.md` proves durable spool preservation, later relay projection, and recipient-visible delivery continuity.

`docs/validation/restart-persistence-slice.md` proves restart persistence across message state, task and escrow state, directory state, and relayed or delivered continuity.

`docs/validation/escrow-settlement-slice.md` proves bounded service-api escrow lifecycle persistence through fund, release, and restart-visible released state.

## Bottom Line
The strongest version of the external critique is strategic, not numerical.

The critique is right that KAMN has too much protocol and process surface relative to the number of undeniable runtime proofs. It is wrong when it presents stale repo-size telemetry and it is now stale on build health.

The correct current reading is:
- KAMN is larger and more implemented than the audit claims.
- KAMN still needs more runtime-proof depth than the repo currently demonstrates.
- There are now multiple real proof anchors on `main`, so the repo can no longer be described honestly as only paper architecture.
