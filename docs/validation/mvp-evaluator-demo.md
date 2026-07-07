# MVP Evaluator Demo

This runbook documents the evaluator-facing KAMN MVP demo command. It packages current bounded local proof surfaces into one local artifact bundle and verifies the proof report's claim boundaries.

## What This Proves
- `make demo-mvp` creates a fresh local demo run directory under `.kamn/demo/<run-id>/`.
- The demo writes `.kamn/demo/latest/proof/report.json` and `.kamn/demo/latest/proof/report.md`.
- The report includes required local MVP claims for runtime startup, authenticated Alice/Bob identities, signed flow, durable state, relay/projection, websocket visibility, and audit/proof export.
- The proof bundle captures the SDK localhost signed demo artifact, service-api working vertical slice output, and service-api websocket output.
- `verify-mvp-demo` rejects missing, malformed, downgraded, dry-run, placeholder, or overclaimed reports.
- Devnet-required mode uses the service API live Solana settlement path when funded devnet keypairs are configured.
- Devnet-required mode writes explicit `NO-GO` evidence when Solana devnet settlement evidence is unavailable.

## What This Does Not Prove
- not production readiness
- not mainnet support
- not generalized exchange
- not broad multi-chain settlement
- not consensus or arbitrary partition tolerance
- not broad bridge finality
- not real economic value; Solana devnet tokens are developer-test tokens only

## Local Demo
Run from a normal checkout:

```bash
make demo-mvp
```

Expected artifacts:

```bash
.kamn/demo/latest/proof/report.json
.kamn/demo/latest/proof/report.md
```

The report links the concrete run artifacts under `.kamn/demo/<run-id>/proof/`, including:

```bash
localhost-signed-demo.json
localhost-signed-demo-output.txt
service-api-vertical-slice-output.txt
service-api-websocket-output.txt
audit-export.json
devnet-settlement-output.txt
```

Verify the generated report:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

The default demo is local-only for runtime, auth, message/task, state, relay, websocket, and audit proof. It does not claim settlement or asset movement success.

## Devnet-Required Demo
Run with the MVP and service API Solana settlement environment configured:

```bash
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL=https://api.devnet.solana.com \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE=/absolute/path/to/devnet-payer.json \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY=<devnet-recipient-pubkey> \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS=1000000 \
KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT=finalized \
make demo-mvp
```

A funded run is expected to return `GO` only after the local service API release path submits and confirms a Solana devnet transfer. The report includes a `devnet_settlement_asset_movement` claim labelled `devnet-backed`, and the proof bundle includes `devnet-settlement-output.txt` with non-secret evidence fields:

```text
devnet_settlement_status=PASS
network=solana:devnet
settlement_tx_signature=<confirmed-devnet-signature>
settlement_commitment=finalized
```

If devnet-backed settlement evidence cannot be produced, the report must remain explicit:

```json
{"status":"NO-GO","devnet_mode":"required"}
```

`NO-GO` is the honest result when devnet funding, transaction submission, confirmation, balance proof, or keypair configuration is unavailable. It must not be downgraded to a local-only settlement pass.

## Claim Labels
Every claim in `claim_matrix` must use exactly one label:

- `real`
- `devnet-backed`
- `local-only`
- `dry-run`
- `placeholder`
- `roadmap`

Required MVP success claims cannot be `dry-run` or `placeholder`.

Any claim involving exchange, escrow, settlement, transfer, lamports, asset movement, or value movement must be `devnet-backed`.

## Verifier Contract
The verifier command:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

returns `{"status":"PASS"}` only when the report schema, artifacts, required local proof claims, claim labels, and value-movement boundaries are valid.
