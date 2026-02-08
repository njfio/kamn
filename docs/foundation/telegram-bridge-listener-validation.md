# Telegram Bridge Listener-Validated Inbound Flow (Issues #222, #223, #587, #614, #662)

This document captures the first implementation slice for Telegram bridge inbound processing with listener validation.

## Scope Delivered
- Added `crates/kamn-core/src/telegram_bridge.rs` with:
  - `TelegramBridgeConfig` for bridge agent DID, authorized listener DIDs, and channel route map.
  - `TelegramInboundRequest` as inbound request envelope with listener DID context.
  - `TelegramBridgeEngine` wrapper over generic bridge adapter flow.
  - typed errors via `TelegramBridgeError`.
- Added Telegram webhook auth and checkpoint safety contracts:
  - `TelegramBridgeConfig.webhook_token` required for ingress auth verification.
  - `TelegramInboundRequest.webhook_token` must match configured token.
  - `TelegramInboundRequest.checkpoint` must be strictly increasing per channel.
  - typed rejection errors: `InvalidWebhookToken`, `NonMonotonicCheckpoint`.
- Added integration tests in `crates/kamn-core/tests/telegram_bridge.rs`.
- Added bridge replay fixture hardening references for low-cost CI coverage:
  - `fixtures/bridge_replay/replay_validation_cases.json`
  - suite subset execution via `scripts/bridge/run_bridge_replay_matrix.sh`
  - changed-adapter selector output `bridge_replay_suites`
- Added Telegram ingress lane entrypoints:
  - `scripts/bridge/run_telegram_ingress_contract_lane.sh`
  - `scripts/bridge/run_telegram_ingress_deep_lane.sh`

## Validation Rules
- Bridge agent DID and listener DIDs must parse as `kamn:did:agent:*`.
- At least one authorized listener DID is required.
- At least one Telegram channel route is required.
- Each route target DID must be valid.
- Inbound processing requires:
  - listener DID must be authorized.
  - webhook token must match configured Telegram auth token.
  - inbound `external_channel_id` must exist in route map.
  - inbound `target_agent_did` must match mapped route target.
  - checkpoint must be monotonic per `external_channel_id`.
  - duplicate replay ingress is rejected deterministically by adapter replay guards.

## Webhook Auth and Checkpoint Safety Contract
- Fast contract lane:
  - `bash scripts/bridge/run_telegram_ingress_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/bridge/run_telegram_ingress_deep_lane.sh`
- Regression policy:
  - forged webhook tokens and replayed/out-of-order checkpoints are rejected (`Regression: #621`).

## Replay Fixture Hardening
- Bridge replay harness validates Telegram replay contracts without running all adapter suites for every bridge diff.
- Fast-gate bridge lane consumes `bridge_replay_suites` and executes changed-adapter subsets first.
- Telegram-focused bridge diffs should run:
  - `--suites bridge_adapter,telegram_bridge`
- Replay fixture corpus includes duplicate replay and malformed ingress classes (`Regression: #587`).

## Processing Flow
- `process_inbound(...)`:
  - validates listener + route constraints.
  - delegates canonical normalization to `BridgeAdapterEngine` with Telegram platform adapter.
- `process_inbound_to_envelope(...)`:
  - runs listener/route validation.
  - converts inbound message into canonical KAMN envelope.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test telegram_bridge
bash scripts/bridge/run_bridge_replay_matrix.sh --fixture fixtures/bridge_replay/replay_validation_cases.json --suites bridge_adapter,telegram_bridge --output-json /tmp/telegram-bridge-replay-report.json
bash scripts/bridge/test_run_telegram_ingress_contract_lane.sh
bash scripts/bridge/run_telegram_ingress_deep_lane.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
