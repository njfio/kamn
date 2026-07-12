# Add One Canonical Agent Transaction Demo Command

## Objective

Provide one evaluator command that validates prerequisites, starts the local
KAMN transaction surface, launches three independent bounded Pi actors, drives
the runtime receipt chain, requires finalized Solana devnet settlement, writes
one proof directory with human and machine reports, verifies it after all child
processes exit, and leaves no child process running on success or failure.

## Inputs/Outputs

The canonical input is:

```bash
KAMN_MVP_AGENT_DRIVER=pi \
KAMN_MVP_DEVNET_MODE=required \
KAMN_MVP_SOLANA_RPC_URL=https://api.devnet.solana.com \
make demo-agent-transaction
```

Required external inputs are the installed `pi` CLI with existing OpenAI Codex
OAuth, the project-local KAMN Pi extension, three external KAMN agent key files,
a funded external Solana devnet payer keypair, recipient pubkey, positive
lamport amount, devnet RPC URL, and finalized commitment. Secrets are referenced
by path and never copied into command output, Pi prompts, child environments, or
proof artifacts.

Success outputs exactly one run directory below `.kamn/demo/`, one unambiguous
`.kamn/demo/latest` pointer, `proof/report.json`, `proof/report.md`, three actor
artifacts, runtime receipts/projections, runtime receipt chain, devnet evidence,
and a concise terminal `GO` summary. Failure outputs one bounded NO-GO report
with a stable reason code and no success claim.

## Boundaries/Non-goals

- Reuse the existing Rust harness, Pi extension, MCP sessions, local KAMN
  service path, devnet settlement path, actor artifacts, runtime chain, report,
  and verifier.
- Make owns only a thin alias. Process supervision and report semantics are
  Rust-owned; Pi owns bounded role decisions; existing tools own transaction
  payloads and authorization.
- Pi processes receive only their role-specific extension tool allowlist and no
  built-in file, shell, edit, write, search, or arbitrary MCP tools.
- Validate every prerequisite before launching actors or creating a task.
- Keep `make demo-mvp` unchanged until three consecutive fresh canonical live
  rehearsals pass.
- Issue #7103 owns independent evaluator/explorer UX. Restart and duplicate-
  transfer hardening beyond existing idempotency belongs to the next issue.
- No mainnet, production custody, web UI, generalized orchestration framework,
  multi-chain settlement, disputes, or committed mutable live artifacts.

## Execution Contract

The Rust supervisor creates one run root and fixed coordination/evidence paths,
starts children in dependency order, captures stdout/stderr to role-specific
logs, and records every child PID. Agent A registers and creates the task; Agent
B registers, accepts, waits for funding, completes, and projects; Agent A waits
for completion, releases finalized devnet settlement, and projects; Agent C
registers and verifies the restricted projection. The exact tool sequence may
be coordinated through existing identifier-only handoff files, never through
authorization-bearing files.

Each actor command uses persistent `pi --mode rpc --no-session --approve --no-extensions`
with the explicit project extension, OpenAI provider/model from validated
configuration, `--no-builtin-tools`, and a role-specific `--tools` allowlist.
The supervisor accepts success only from process exit status plus strict actor,
chain, report, and canonical verifier evidence; Pi prose is diagnostic only.

On any error or signal, the supervisor terminates all known Pi/MCP/runtime
children, waits for them, writes NO-GO once, and exits non-zero. Cleanup is
idempotent. A stale prior `latest` pointer is never reported as the current run.

## Claim Labels

- Local runtime identity, task, durable state, MCP receipts, projections,
  websocket/event visibility, and audit export are `local` or `local-only`.
- Confirmed Solana transfer and finalized signature evidence are
  `devnet-backed`.
- A dry-run is `dry-run` and can never produce GO.
- Placeholder signatures, fixture-only commands, and in-memory-only value
  movement are `placeholder` and can never produce GO.
- Mainnet, custody, production availability, disputes, and generalized
  marketplace behavior are `not-claimed`.

## Failure Modes

- Unsupported/missing agent driver: `AGENT_TRANSACTION_DRIVER_INVALID`.
- Pi missing, auth unavailable, model unavailable, or extension missing:
  `AGENT_TRANSACTION_PI_PREFLIGHT_FAILED`.
- Partial/missing agent keys or unsafe secret path:
  `AGENT_TRANSACTION_AGENT_CONFIG_INVALID`.
- Missing/invalid devnet payer, recipient, amount, RPC, or commitment:
  `AGENT_TRANSACTION_DEVNET_CONFIG_INVALID`.
- Existing output/latest ambiguity: `AGENT_TRANSACTION_OUTPUT_CONFLICT`.
- Child spawn, timeout, non-zero exit, or malformed output:
  `AGENT_TRANSACTION_CHILD_FAILED`.
- Child survives cleanup: `AGENT_TRANSACTION_CLEANUP_FAILED`.
- Dry-run, placeholder, copied, non-finalized, or missing settlement evidence:
  `AGENT_TRANSACTION_SETTLEMENT_INVALID`.
- Missing/mismatched actor, chain, report, or verifier evidence:
  `AGENT_TRANSACTION_PROOF_INVALID`.

Every failure hard-fails, writes NO-GO when the output root is safely available,
and never reuses a prior GO report.

## Acceptance Criteria

- [x] The documented canonical environment plus `make demo-agent-transaction`
      invokes the Rust-owned supervisor.
- [x] Driver, Pi/auth/model/extension, agent keys, and devnet configuration are
      validated before any task mutation.
- [x] Three independent bounded Pi processes and their MCP children start and
      stop automatically.
- [x] Actor role tool allowlists exclude built-in and cross-role capabilities.
- [x] Any child failure or signal writes NO-GO and leaves no known child alive.
- [x] Dry-run, placeholder, copied, missing, or non-finalized settlement cannot
      produce GO.
- [x] Success writes exactly one proof directory and one unambiguous latest
      pointer containing reports, actors, receipts, projections, and devnet
      evidence.
- [x] The report labels local, devnet-backed, dry-run, placeholder, roadmap, and
      not-claimed surfaces honestly.
- [x] Canonical verification passes after every demo process exits.
- [x] No secret value or mutable live artifact is committed or emitted.
- [x] Formatting, strict clippy, focused integration tests, and `make check`
      pass.

## Files To Touch

- `Makefile`
- `crates/kamn-e2e-harness/src/lib.rs`
- focused `crates/kamn-e2e-harness/src/agent_transaction_demo*.rs`
- focused `crates/kamn-e2e-harness/tests/agent_transaction_demo*.rs`
- minimal `.pi/extensions/kamn-mvp/` surfaces only if an existing bounded tool
  cannot be invoked non-interactively
- `docs/validation/mvp-evaluator-demo.md`

The harness adds only the existing local `kamn-sdk` workspace crate for exact
key-bound DID derivation. Shell LOC remains a thin Make alias; the issue
estimate is revised toward Rust ownership rather than adding a shell supervisor.

## Error Semantics

Interior Rust functions return stable reason-code-prefixed `String` errors and
do not log. The command entrypoint catches once, writes the terminal/report
decision once, performs cleanup, and exits non-zero on NO-GO. Child logs redact
secret-bearing arguments and environments. Cleanup errors are preserved and
cannot be hidden by an earlier actor or settlement error.

## Test Plan

RED:

- Missing Pi/auth/model or partial devnet configuration reaches task work.
- Role command construction enables built-in or cross-role tools.
- One actor exits and a tracked child remains alive.
- Dry-run, placeholder, or non-finalized settlement produces GO.
- A failure leaves a stale latest pointer or multiple candidate proof roots.
- A successful-looking report is accepted before all children exit.

GREEN:

- Add strict preflight configuration parsing and safe-path validation.
- Add Rust child supervisor with bounded Pi command construction, timeout,
  signal/error cleanup, wait, and redacted logs.
- Reuse actor files and the #7101 runtime chain as configured `demo-mvp` inputs.
- Run canonical verification only after all actor/runtime children exit.
- Write one GO/NO-GO result and atomically update latest only for this run.

REFACTOR:

- Keep config, preflight, command construction, process supervision, proof
  finalization, and reporting separate.
- Keep files below 200 lines and functions below 25 lines.
- Reuse existing strict path, process, evidence, and report helpers.

INTEGRATION:

- Run deterministic stub-child contracts for preflight, cleanup, NO-GO, and
  single-output semantics.
- Run a bounded local three-Pi rehearsal with existing Codex OAuth.
- Run required Solana devnet settlement or record explicit external NO-GO.
- Verify reports after all children exit.
- Run formatting, strict workspace clippy, `make check`, focused suites, and
  issue shell-surface accounting.

## Rollback

Preserve spec, RED, GREEN, REFACTOR, and integration commits. The Make alias and
new command can be reverted without changing existing `demo-mvp`, Pi tools,
runtime transaction semantics, actor artifacts, or report compatibility.
