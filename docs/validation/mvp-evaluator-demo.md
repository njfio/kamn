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
three-agent-transcript.json
```

Verify the generated report:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

The default demo is local-only for runtime, auth, message/task, state, relay, websocket, and audit proof. It does not claim settlement or asset movement success.

## Optional Pi Agent Harness
KAMN includes a project-local Pi extension at `.pi/extensions/kamn-mvp/index.ts`.
It registers named local proof tools:

- `kamn_verify_mvp_report`
- `kamn_inspect_mvp_report_boundaries`
- `kamn_agent_a_register`
- `kamn_agent_a_invoke_transaction`
- `kamn_agent_b_register`
- `kamn_agent_b_accept_task`
- `kamn_agent_c_verify_three_agent_proof`
- `kamn_write_agent_harness_evidence`
- `kamn_run_demo_mvp_with_agent_evidence`

Run Pi with the extension and the Codex-backed model:

```bash
env -u OPENAI_API_KEY pi \
  --model openai-codex/gpt-5.5 \
  --thinking medium \
  --extension .pi/extensions/kamn-mvp/index.ts \
  --no-builtin-tools \
  --tools kamn_verify_mvp_report,kamn_inspect_mvp_report_boundaries,kamn_agent_a_register,kamn_agent_a_invoke_transaction,kamn_agent_b_register,kamn_agent_b_accept_task,kamn_agent_c_verify_three_agent_proof,kamn_write_agent_harness_evidence,kamn_run_demo_mvp_with_agent_evidence \
  --approve \
  --no-session \
  -p "Use only the KAMN tools. Verify the current report, inspect claim boundaries, call the Agent A register tool, Agent A invoke transaction tool, Agent B register tool, Agent B accept task tool, and Agent C verify proof tool against the current report, then write /tmp/kamn-pi-mcp-agent-harness-evidence.json and verify the same report with agentHarnessEvidencePath set to that evidence path."
```

The final Pi verifier tool call executes the equivalent canonical command:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo \
  --report .kamn/demo/latest/proof/report.json \
  --agent-harness-evidence /tmp/kamn-pi-mcp-agent-harness-evidence.json
```

In this flow one canonical report remains unchanged. The verifier reads the Pi
artifact separately and fails if its report path, claim boundaries, actor tool
receipts, or canonical observation receipt artifacts/digests disagree with that
report. This does not rerun settlement or submit another devnet transaction.

When the Pi tool path writes evidence, the proof artifact records
`execution_surface:"pi-extension-tools"` and the final report may include
`mcp_agent_harness_verification`. This proves that Pi extension tools can drive
the KAMN MVP proof path. For devnet-backed three-agent reports, Pi must call the
actor tools before `kamn_write_agent_harness_evidence`; otherwise evidence
writing fails loudly instead of manufacturing success. The resulting artifact
includes `three_agent_actor_tool_receipts` bound to the report path, actor,
action, sequence, view scope, view artifact, and view digest.
It also includes `three_agent_actor_observation_receipts` that name the
canonical `agent_a_observation_receipt_digest`,
`agent_b_observation_receipt_digest`, and
`agent_c_verifier_observation_receipt_digest` references from the verified
report. Agent C's verifier receipt evidence remains restricted-public and must
not expose participant-private digest or raw private payload markers.

The evidence also records a `three_agent_boundary` summary derived from the
verified report: local-only reports are marked `NOT_PRESENT`, while reports with
`three_agent_escrow_verification` must preserve the report's claim status, label,
participant private-field counts, verifier zero-private-field count, redaction
marker, and absence of a verifier private digest. This does not prove generic Pi MCP protocol support, and it does not upgrade local-only or dry-run settlement into MVP success.

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

Successful devnet-backed reports also include
`three-agent-transcript.json`. That artifact records the local proof transcript
for Agent A registration, Agent B registration, Agent A task or transaction
invocation, Agent B acceptance, escrow funding and release, and Agent C
restricted-public verification. It links those steps to the report's Solana
devnet settlement signature, amount, payer, recipient, and finalized commitment.
The transcript is local proof evidence; the settlement or asset-movement claim
remains `devnet-backed`, and raw participant-private payloads stay redacted.

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
