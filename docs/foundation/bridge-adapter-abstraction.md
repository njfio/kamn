# Bridge Adapter Abstraction (Issues #130, #131, #546, #587, #589, #590, #612)

This document describes the first implementation slice for bridge adapter abstraction aligned to PRD section 10.1 and story #34.

## Scope Delivered
- Added a common bridge adapter contract for inbound normalization and outbound translation:
  - `BridgeAdapter` trait
  - `BridgePolicyHook` trait
  - `BridgeAdapterEngine` orchestration layer
- Added deterministic bridge data models:
  - `BridgeInboundEnvelope`
  - `NormalizedInboundMessage`
  - `BridgeOutboundRequest`
  - `BridgeOutboundEnvelope`
- Added first concrete implementation:
  - `PassThroughBridgeAdapter`
  - `AllowAllBridgePolicy`
- Added inbound-to-canonical-envelope projection path so bridge ingress can be validated using existing `CanonicalMessageEnvelope` schema checks.
- Added integration tests covering deterministic behavior, policy denial, and regression guard for outbound request ID mutation.
- Added shared bridge replay fixture corpus and matrix harness:
  - `fixtures/bridge_replay/replay_validation_cases.json`
  - `scripts/bridge/run_bridge_replay_case.sh`
  - `scripts/bridge/run_bridge_replay_matrix.sh`
- Added CI scope routing for bridge-only diffs:
  - selector output `run_bridge_replay_harness`
  - selector output `bridge_replay_suites` to run changed adapter subsets.
  - fast-gate bridge harness command runs only for bridge-related path changes and passes selected suites.

## Design Notes
- Inbound flow:
  - Validate external envelope fields and target DID.
  - Enforce timestamp freshness by requiring `observed_at_unix - received_at_unix <= max_inbound_age_secs`.
  - Normalize external payload into deterministic `bridge_message_id` (`<platform>:<external_message_id>`).
  - Apply policy hook before returning normalized message.
  - Duplicate inbound message IDs are rejected with `DuplicateInboundMessageId`.
  - Stale inbound messages are rejected with `StaleInboundMessage`.
- Outbound flow:
  - Validate request fields and sender DID.
  - Apply policy hook before translation.
  - Enforce request ID stability to prevent adapter-side mutation.
  - Reject duplicate outbound request IDs with `DuplicateOutboundRequestId`.
- Cross-module integration:
  - `process_inbound_to_envelope(...)` maps normalized bridge ingress into `CanonicalMessageEnvelope` and runs envelope validation before returning.
  - Bridge engine wrappers apply route/listener checks while keeping single-pass inbound projection so replay guards run exactly once per logical projection flow.
- Replay fixture coverage classes:
  - duplicate replay: Telegram, Discord, and cross-chain inbound projection replay rejections.
  - stale ingress: stale bridge inbound message rejection.
  - malformed ingress: route-target mismatch and unknown route rejection paths.
  - signature-failure: unauthorized approver signatures are rejected for Discord and cross-chain outbound quorum flows.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test bridge_adapter
bash scripts/bridge/run_bridge_replay_matrix.sh --fixture fixtures/bridge_replay/replay_validation_cases.json --suites bridge_adapter,telegram_bridge --output-json /tmp/bridge-replay-report.json
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add platform-specific adapters (Telegram/Discord/Slack) with real credential and rate-limit hooks.
- Replace static allow-all policy with channel and capability-aware policy implementations.
- duplicate inbound event is rejected (`Regression: #423`).
- duplicate outbound request is rejected (`Regression: #433`).
- stale inbound event beyond freshness window is rejected (`Regression: #546`).
- first inbound-to-envelope projection does not self-trigger duplicate replay rejection (`Regression: #438`).
- cross-chain inbound projection also preserves single-pass replay safety (`Regression: #443`).
- bridge replay fixture matrix guards duplicate/stale/malformed replay behavior across adapters (`Regression: #587`).
- bridge replay fixture matrix includes signature-failure class coverage and adapter subset execution (`Regression: #587`).
