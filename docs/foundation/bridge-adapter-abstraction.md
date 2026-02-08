# Bridge Adapter Abstraction (Issues #130, #131)

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

## Design Notes
- Inbound flow:
  - Validate external envelope fields and target DID.
  - Normalize external payload into deterministic `bridge_message_id` (`<platform>:<external_message_id>`).
  - Apply policy hook before returning normalized message.
  - Duplicate inbound message IDs are rejected with `DuplicateInboundMessageId`.
- Outbound flow:
  - Validate request fields and sender DID.
  - Apply policy hook before translation.
  - Enforce request ID stability to prevent adapter-side mutation.
  - Reject duplicate outbound request IDs with `DuplicateOutboundRequestId`.
- Cross-module integration:
  - `process_inbound_to_envelope(...)` maps normalized bridge ingress into `CanonicalMessageEnvelope` and runs envelope validation before returning.
  - Bridge engine wrappers apply route/listener checks while keeping single-pass inbound projection so replay guards run exactly once per logical projection flow.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test bridge_adapter
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add platform-specific adapters (Telegram/Discord/Slack) with real credential and rate-limit hooks.
- Replace static allow-all policy with channel and capability-aware policy implementations.
- duplicate inbound event is rejected (`Regression: #423`).
- duplicate outbound request is rejected (`Regression: #433`).
- first inbound-to-envelope projection does not self-trigger duplicate replay rejection (`Regression: #438`).
- cross-chain inbound projection also preserves single-pass replay safety (`Regression: #443`).
