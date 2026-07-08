# MVP Evaluator Rehearsal Evidence - 2026-07-07

Issue: #7043
Spec: `specs/7043-mvp-evaluator-rehearsal.md`
Branch: `7043-mvp-evaluator-rehearsal`
Base checked for rehearsal: `origin/main` at `afc507a4a9c5`

## Clean Worktree

```text
/tmp/kamn-mvp-evaluator-7043-20260707-183715
```

The worktree was detached at `origin/main` and remained clean before the
rehearsal-owned `.kamn/` demo artifacts were produced.

No README, runbook, or env-example change was required. The existing root
README and `docs/validation/mvp-evaluator-demo.md` were enough to run the local
and funded devnet paths.

## Local-Only Demo

Command:

```bash
make demo-mvp
```

Result:

- status: `GO`
- run id: `run-57898-1783463882414`
- report: `.kamn/demo/run-57898-1783463882414/proof/report.json`
- devnet mode: `optional`
- verifier:
  `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report /tmp/kamn-mvp-evaluator-7043-20260707-183715/.kamn/demo/run-57898-1783463882414/proof/report.json`
- verifier result:
  `{"status":"PASS","report":"/tmp/kamn-mvp-evaluator-7043-20260707-183715/.kamn/demo/run-57898-1783463882414/proof/report.json"}`

Claim Boundaries:

- `local_runtime_startup`: `real`, required, `PASS`
- `authenticated_agent_identities`: `local-only`, required, `PASS`
- `signed_message_or_task_flow`: `local-only`, required, `PASS`
- `durable_state_written`: `local-only`, required, `PASS`
- `relay_projection_visible`: `local-only`, required, `PASS`
- `websocket_event_visibility`: `local-only`, required, `PASS`
- `audit_proof_export`: `local-only`, required, `PASS`
- `production_readiness`: `roadmap`, optional, `NOT_CLAIMED`

The local-only run did not claim settlement, escrow execution, asset movement,
or value movement success.

## Devnet-Required Demo

```bash
set -a
. /Users/n/RustroverProjects/kamn/.kamn/devnet/mvp-demo-devnet.env
set +a
make demo-mvp
```

The env file is local and ignored. Its contents were not printed, copied, or
committed.

Result:

- status: `GO`
- run id: `run-73263-1783464071530`
- report: `.kamn/demo/run-73263-1783464071530/proof/report.json`
- devnet mode: `required`
- verifier:
  `cargo run -p kamn-e2e-harness -- verify-mvp-demo --report /tmp/kamn-mvp-evaluator-7043-20260707-183715/.kamn/demo/run-73263-1783464071530/proof/report.json`
- verifier result:
  `{"status":"PASS","report":"/tmp/kamn-mvp-evaluator-7043-20260707-183715/.kamn/demo/run-73263-1783464071530/proof/report.json"}`

Devnet-backed settlement evidence:

- claim: `devnet_settlement_asset_movement`
- label: `devnet-backed`
- network: `solana:devnet`
- RPC URL: `https://api.devnet.solana.com`
- payer: `2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T`
- recipient: `FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe`
- lamports: `1000000`
- signature:
  `2dWRAChLFzqAFxpNPYAb6ZGkP6Ms6yrLJm6ZGYXG7XmM8rXy2Emmy8myhva6gtCNbpkusCrCHfGa14oR7PamHGss`
- commitment: `finalized`
- payer balance: `2492965000` before, `2491960000` after
- recipient balance: `2507000000` before, `2508000000` after
- persisted settlement signature matches the Solana signature

Independent Solana confirmation:

```bash
solana confirm -v --url https://api.devnet.solana.com \
  2dWRAChLFzqAFxpNPYAb6ZGkP6Ms6yrLJm6ZGYXG7XmM8rXy2Emmy8myhva6gtCNbpkusCrCHfGa14oR7PamHGss
```

The transaction executed in slot `474701354` with block time
`2026-07-07T18:41:31-04:00`, status `Ok`, one system transfer of
`1000000` lamports, and finality `Finalized`.

## Secret Scan

Command pattern:

```bash
rg -n 'KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE|mvp-settlement-payer|BEGIN .*PRIVATE|\[[0-9]+,\s*[0-9]+,\s*[0-9]+,\s*[0-9]+' \
  .kamn/demo/run-73263-1783464071530/proof \
  .kamn/demo/run-73263-1783464071530/state \
  .kamn/demo/run-73263-1783464071530/events
```

Result: `rg` exit code `1`, no matches.

No private key, keypair JSON, env file, or private credential content was recorded
in the inspected proof, state, or event artifacts.

## Agent Harness Evaluation

### Pi Harness

Official sources checked:

- `https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/quickstart.md`
- `https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/providers.md`
- `https://github.com/earendil-works/pi/blob/main/packages/coding-agent/docs/usage.md`

- `pi` and `pi-acp` were not already installed on PATH.
- `npx -y @earendil-works/pi-coding-agent --version` returned `0.80.3`.
- `npx -y @earendil-works/pi-coding-agent --help` confirmed non-interactive
  `-p`, `--provider`, `--model`, `--tools`, `--approve`, and context-file
  loading support.
- `~/.pi/agent/auth.json` was missing.
- `OPENAI_API_KEY` was present, but a bounded Pi run using
  `--provider openai --model gpt-4o-mini --tools read,bash,grep,find,ls`
  failed with OpenAI `401 invalid_api_key`.

Pi result: blocked by local auth. No committed secret was added, and no repo file
was edited by Pi.

### Goose Harness

Official sources checked:

- `https://goose-docs.ai/docs/guides/acp-providers/`
- `https://github.com/aaif-goose/goose`

- `brew info block-goose-cli` reported stable `1.41.0`.
- `brew install block-goose-cli` installed `/opt/homebrew/bin/goose`.
- `npm install -g @agentclientprotocol/codex-acp` installed
  `/Users/n/.nvm/versions/node/v22.22.0/bin/codex-acp`.
- A minimal Goose/Codex ACP prompt succeeded with `GOOSE_MODE=approve`.
- The full evaluator prompt reached `make demo-mvp` but failed before execution
  because `approve` mode requires an interactive terminal in non-interactive
  Goose runs.
- `GOOSE_MODE=auto` and `GOOSE_MODE=smart-approve` failed before command
  execution because Goose requested `full-access`, while the current Codex ACP
  adapter advertises `read-only`, `agent`, and `agent-full-access`.
- Installing the deprecated Goose-documented `@zed-industries/codex-acp`
  adapter was not forced because it would overwrite the current `codex-acp`
  binary globally.

Goose result: blocked by non-interactive approval/mode mismatch, not by KAMN.

## Current-Branch Demo

```bash
make demo-mvp
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

Result: run id `run-30651-1783468978384`, status `GO`, devnet mode
`optional`, verifier `PASS`.

## Local Gate Status

- `cargo fmt --check`: `PASS`.
- `cargo test -p kamn-e2e-harness --test mvp_evaluator_rehearsal_docs_contract -- --nocapture`: `PASS`.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: blocked locally. Two `kamn-core` clippy-driver processes repeatedly wedged with no diagnostics or CPU progress.
- `make check`: blocked for the same strict clippy wedge; it was interrupted rather than weakened.

## Remaining Risks

- The MVP demo is credible as a local and Solana-devnet evaluator path, not as a
  production-readiness claim.
- Pi can likely be re-attempted after a valid Pi OAuth login or valid OpenAI API
  key is available.
- Goose can likely be re-attempted after its Codex ACP provider mode mapping is
  configured or the adapter mismatch is resolved without globally overwriting a
  working adapter.
- The funded devnet path moves Solana devnet lamports only. It does not prove
  mainnet settlement, generalized exchange, or real economic value.
