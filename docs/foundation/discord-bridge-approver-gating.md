# Discord Bridge Approver-Gated Outbound Flow (Issues #220, #221, #587, #614)

This document captures the first implementation slice for Discord bridge processing with approver quorum gating on outbound actions.

## Scope Delivered
- Added `crates/kamn-core/src/discord_bridge.rs` with:
  - `DiscordBridgeConfig` for bridge DID, listener allowlist, approver allowlist, required approval threshold, and route map.
  - `DiscordInboundRequest` and `DiscordBridgeEngine` for listener-validated inbound normalization.
  - `process_outbound_with_approvals(...)` for approver-gated outbound dispatch.
  - `DiscordOutboundApproval` and `DiscordOutboundDispatch` to preserve deterministic approval metadata for auditing.
  - typed error handling via `DiscordBridgeError`.
- Added integration tests in `crates/kamn-core/tests/discord_bridge.rs`.
- Added bridge replay fixture hardening references for low-cost CI coverage:
  - `fixtures/bridge_replay/replay_validation_cases.json`
  - suite subset execution via `scripts/bridge/run_bridge_replay_matrix.sh`
  - changed-adapter selector output `bridge_replay_suites`

## Validation Rules
- Bridge/listener/approver DIDs must parse as `kamn:did:agent:*`.
- At least one authorized listener and approver DID is required.
- `required_approvals` must be between `1` and approver allowlist size.
- At least one route entry is required and route target DIDs must be valid.
- Outbound processing requires:
  - destination channel mapped in route map.
  - approver set contains no duplicates.
  - all approvers are authorized.
  - provided approvals satisfy quorum threshold.
  - unauthorized approver signatures are treated as signature-failure fixture class outcomes.

## Replay Fixture Hardening
- Bridge replay harness validates Discord replay contracts without requiring full bridge suite execution for every diff.
- Fast-gate bridge lane consumes `bridge_replay_suites` and runs adapter subsets first.
- Discord-focused bridge diffs should run:
  - `--suites bridge_adapter,discord_bridge`
- Replay fixture coverage includes signature-failure class checks for unauthorized approver rejection (`Regression: #587`).

## Processing Flow
- `process_inbound(...)`:
  - validates listener identity and route target mapping.
  - normalizes inbound payload through `BridgeAdapterEngine` with Discord platform adapter.
- `process_inbound_to_envelope(...)`:
  - preserves listener/route checks.
  - projects to canonical KAMN message envelope.
- `process_outbound_with_approvals(...)`:
  - enforces channel routing + approver quorum checks.
  - delegates outbound translation to `BridgeAdapterEngine`.
  - returns translated payload plus approval metadata for audit trails.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test discord_bridge
bash scripts/bridge/run_bridge_replay_matrix.sh --fixture fixtures/bridge_replay/replay_validation_cases.json --suites bridge_adapter,discord_bridge --output-json /tmp/discord-bridge-replay-report.json
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
