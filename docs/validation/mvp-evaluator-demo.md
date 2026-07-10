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

When `--agent-harness-evidence` is supplied directly to `verify-mvp-demo`, the
same checks run without adding `mcp_agent_harness_verification` to the report.
The verifier result names the separately validated evidence artifact instead.

The underlying `kamn-mcp-server` `register` tool is service-backed: it signs an
`agents:write` request, calls `POST /v1/agents/register`, and returns the
persisted profile. The similarly named project-local Pi actor receipt tool
still records report-bound evaluator evidence; it is not yet wired to invoke
the live MCP registration process. Keep those two surfaces distinct when
describing the current demo.

### Live Pi Identity Check

Pi can also prove local-only identity durability through a persistent live MCP process. In one terminal, build the binaries, create a disposable Agent A key, and start the local KAMN service:

```bash
cargo build -p kamn-mcp-server -p kamn-node
openssl rand -hex 32 > /tmp/kamn-pi-agent-a.key
KAMN_SERVICE_API_TLS_MODE=disabled target/debug/kamn-node \
  --runtime-mode api \
  --role processor \
  --api-bind 127.0.0.1:18278 \
  --api-max-requests 1000 \
  --api-idle-timeout-ms 600000 \
  --storage-dir /tmp/kamn-node-pi-live
```

In a second terminal, run the two live Pi tools:

```bash
KAMN_MVP_LIVE_MCP_BINARY=target/debug/kamn-mcp-server \
KAMN_MVP_LIVE_MCP_ENDPOINT=http://127.0.0.1:18278 \
KAMN_MVP_LIVE_MCP_AGENT_A_NAME=pi-agent-a \
KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE=/tmp/kamn-pi-agent-a.key \
env -u OPENAI_API_KEY pi \
  --model openai-codex/gpt-5.5 \
  --thinking medium \
  --extension .pi/extensions/kamn-mvp/index.ts \
  --no-builtin-tools \
  --tools kamn_live_agent_a_register,kamn_live_agent_a_query_profile \
  --approve \
  --no-session \
  -p "Use only the KAMN tools. Register Agent A, then query the same durable Agent A profile. Report the claim boundary exactly."
```

`kamn_live_agent_a_register` starts `kamn-mcp-server` lazily. `kamn_live_agent_a_query_profile` reuses that process, preserving its authenticated request nonce and querying the DID returned by registration. The extension passes the key-file path to the child process but does not read or return key contents; Pi session shutdown terminates the child. Only KAMN-prefixed configuration and basic process variables are forwarded to the child; Pi and OpenAI credentials are not forwarded.

A passing node log shows `POST /v1/agents/register` with request nonce `1` and status `201`, followed by `GET /v1/agents/<same-did>` with request nonce `2` and status `200`.

This proves local-only identity durability through the live service API. It does not prove task, escrow, settlement, or asset movement, and it does not replace the devnet-required proof for any value-movement claim.

### Live Pi Two-Agent Task Check

The next bounded check uses two independently authenticated MCP children for a real local-only task lifecycle. Keep the node from the identity check running, create a second disposable key, and run Pi with both agent configurations:

```bash
openssl rand -hex 32 > /tmp/kamn-pi-agent-b.key

KAMN_MVP_LIVE_MCP_BINARY=target/debug/kamn-mcp-server \
KAMN_MVP_LIVE_MCP_ENDPOINT=http://127.0.0.1:18278 \
KAMN_MVP_LIVE_MCP_AGENT_A_NAME=pi-agent-a \
KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE=/tmp/kamn-pi-agent-a.key \
KAMN_MVP_LIVE_MCP_AGENT_B_NAME=pi-agent-b \
KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE=/tmp/kamn-pi-agent-b.key \
env -u OPENAI_API_KEY pi \
  --model openai-codex/gpt-5.5 \
  --thinking medium \
  --extension .pi/extensions/kamn-mvp/index.ts \
  --no-builtin-tools \
  --tools kamn_live_agent_a_register,kamn_live_agent_b_register,kamn_live_agent_a_create_task,kamn_live_agent_b_accept_task,kamn_live_agent_a_query_task,kamn_live_agent_b_query_task \
  --approve \
  --no-session \
  -p "Use only the KAMN tools. Register Agent A and Agent B. As Agent A create a task titled 'Review KAMN proof' with description 'Validate the local two-agent task lifecycle'. As Agent B accept it. Query the accepted task as Agent A and Agent B. Report the claim boundary exactly."
```

A passing run returns distinct Agent A and Agent B DIDs, one shared task ID, `submitted` from creation, and `accepted` from acceptance and both queries. Node logs show independent nonce domains: Agent A uses request nonces `1`, `2`, and `3` for register/create/query; Agent B independently uses `1`, `2`, and `3` for register/accept/query. Both registrations and task creation return `201`; acceptance and both task queries return `200`.

This is a real local-only task lifecycle backed by the service task store. It does not prove escrow, settlement, asset movement, third-party verification, or restart durability. Those claims require their own proof slices, and any eventual value-movement claim must remain devnet-backed.

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
