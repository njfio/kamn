# 6991-live-solana-bridge-dispatch

## Objective
Replace the synthetic service-api bridge forwarding placeholder values with bounded live Solana-backed dispatch evidence on current `main`. The bounded live lane must derive forward evidence from a real Solana devnet-backed interaction and persist that evidence through the existing service-api bridge forward path, without claiming settlement or production readiness.

## Inputs/Outputs
- Inputs:
  - existing service-api bridge submit/forward/query routes
  - existing Solana live proof lane from `#6989`
  - startup configuration for the bounded live Solana bridge dispatch lane
- Outputs:
  - non-synthetic `target_message_id` and `forward_tx_hash` values on the bounded live forward path
  - persisted bridge state carrying live-backed evidence instead of placeholders
  - one integration test exercising the real forward path
  - one bounded runbook under `docs/validation/`

## Boundaries/Non-goals
- Do not claim external settlement.
- Do not claim production readiness.
- Do not weaken the existing bounded Solana proof slices.
- Do not keep placeholder `msg-bridge-target-*` or `sha256:bridge-forwarded-*` values on the claimed live path.
- Do not add a Solana SDK dependency unless the existing toolchain proves insufficient.

## Failure modes
- The live path still persists placeholder `target_message_id` or `forward_tx_hash` values.
- The forward path claims live Solana backing without performing a real Solana-backed interaction.
- Missing Solana live configuration fails late at request time instead of at startup.
- The runbook overstates the proof as settlement or full bridge finality.

## Acceptance criteria (testable booleans)
- [x] A bounded live Solana bridge-forward path exists behind startup-validated configuration.
- [x] The live path persists non-synthetic bridge evidence in `target_message_id` and `forward_tx_hash`.
- [x] A hard-fail integration test proves the live path does not emit placeholder bridge values.
- [x] Restart-visible bridge state preserves the live-backed evidence.
- [x] A bounded validation runbook documents what the slice proves and what it does not prove.

## Files to touch
- `specs/6991-live-solana-bridge-dispatch.md`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/content_bridge_ops.rs`
- `crates/kamn-node/src/service_api_endpoint/payload.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/`
- `docs/validation/`
- `docs/validation/current-proven-runtime-slices.md`

## Error semantics
- Missing or invalid live Solana bridge configuration must fail at startup.
- Live bridge dispatch failures must return explicit typed errors; no fallback to synthetic placeholder evidence.
- Persistence failures remain hard failures.

## Test plan
- Phase 3 red tests: add a bridge-forward integration test that asserts placeholder target ids and placeholder forward hashes are rejected on the live Solana-backed path.
- Green by implementing the bounded live bridge forward path and updating persisted bridge state.
- Re-run bridge forward integration, restart persistence, and touched-Rust checks before publish.

## Deviations
- The bounded live lane uses the checked-in live Solana proof runner from `#6989` as its real Solana-backed evidence source instead of introducing a second Solana RPC client into `kamn-node`.
- The bridge restart contract file was split during refactor so the touched-Rust file-size gate stayed green after the live-path tests were added.

## Final evidence
- `cargo test -p kamn-node --test live_solana_bridge_dispatch_slice_contract -- --nocapture`
- `cargo test -p kamn-node --bin kamn-node 'main_tests::service_api_endpoint_tests::bridge_persistence_restart_contract_tests::live_bridge_contract_tests::integration_service_api_endpoint_live_bridge_forward_path_rejects_placeholder_evidence' -- --exact --nocapture`
- `cargo test -p kamn-node --bin kamn-node 'main_tests::service_api_endpoint_tests::bridge_persistence_restart_contract_tests::live_bridge_contract_tests::integration_service_api_endpoint_live_bridge_forward_evidence_survives_restart' -- --exact --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6991-touched-size.json`
