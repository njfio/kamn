# Anti-Spam Controls with Per-Agent Limits and Anti-Sybil Deposits (Issue #186)

This document captures the first implementation slice for anti-spam controls and anti-sybil deposit enforcement.

## Scope Delivered
- Added `crates/kamn-core/src/anti_spam.rs` with:
  - `AntiSpamEngine` evaluation pipeline.
  - `AntiSpamConfig` policy thresholds.
  - `AntiSpamDecision` and `AntiSpamRejection` outcomes.
  - `AntiSpamTelemetry` counters for abuse tuning.
  - `AntiSpamError` typed validation failures.
- Added tests in `crates/kamn-core/tests/anti_spam_controls.rs`.

## Enforcement Rules
- Deposit gate:
  - sender deposit must be at least `minimum_sybil_deposit`.
- Per-agent rate limit:
  - each sender can emit up to `max_messages_per_window` within `window_seconds`.
- Suspension policy:
  - repeated rate-limit violations trigger temporary sender suspension.
- Replay/spam guard:
  - duplicate message IDs are rejected deterministically.

## Telemetry Surface
`AntiSpamTelemetry` tracks:
- total processed requests
- accepted requests
- rejected due to insufficient deposit
- rejected due to rate limit
- rejected due to suspension
- rejected due to duplicate message ID

## Validation and Error Handling
- Config rejects zero/invalid thresholds.
- Sender DID must use `kamn:did:agent:*` format.
- Empty message IDs are rejected.
- Invalid inputs produce explicit typed errors.

## Fast and Cost-Effective Validation
This slice uses focused deterministic tests for PR fast gates:

```bash
cargo test -p kamn-core --test anti_spam_controls
```

No long-running integration harness is required for this stage.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test anti_spam_controls
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core --test anti_spam_controls_docs
cargo test -p kamn-core
```
