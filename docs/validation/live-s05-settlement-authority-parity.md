# Live S-05 Settlement Authority Parity

## Claim Boundary

The deterministic `authoritative_settlement_entrypoint_parity` test proves that
the real SDK, CLI, and MCP dispatch surfaces preserve one escrow, one
idempotency key, byte-identical normalized authority, and one deduplicated
submission at a stateful service boundary. It does not prove funded devnet
execution.

Funded proof exists only when the ignored-live test completes and writes
`.kamn/e2e/live-s05-authority-parity/report.json`. An absent report is
`NOT_EXECUTED`, not `PASS`.

## Capture Contract

Set `KAMN_E2E_S05_AUTHORITY_PARITY_CAPTURE` to a JSON capture from one live
escrow release retried through SDK, CLI, and MCP. The capture must contain:

- `escrow_id`, `actor_did`, and `idempotency_key`
- `attempts.sdk`, `attempts.cli`, and `attempts.mcp` raw response objects
- `settlement_submissions` from
  `settlement_intents[escrow_id].submission_attempt_count`
- payer and recipient balances before and after the release attempts
- `transaction_signature`
- `authoritative_rpc_artifact`, pointing to finalized successful RPC JSON

The three attempts must use the same live endpoint and release payload. Capture
balances before the SDK attempt and after the MCP attempt. Do not include
private keys, bearer tokens, or environment values in the capture.

## Verification

```bash
cargo test -p kamn-e2e-harness \
  --test live_s05_settlement_authority_parity \
  -- --ignored --exact integration_live_s05_authority_parity_capture_is_complete
```

The test rejects missing or partial authority, identity drift, receipt-chain
tampering, cross-resource reuse, conflicting retries, duplicate submissions,
missing balance movement, and non-finalized RPC evidence.

## External Peer Boundary

This proof does not alter the external x402 result. That lane remains
FAIL/BLOCKED until the peer supplies the request digest, nonce, challenge ID,
expiry, approval linkage, and authoritative settlement receipt fields.
