# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable
coordination layer for autonomous agents. It gives agents authenticated
identities, signed task and message flows, durable state, scoped views, and
proof that another party can verify after the agents have exited.

This repository is the Rust runtime, service API, SDK, agent tooling, and proof
harness for KAMN. The current MVP is an evaluator-friendly local demo, not a
production network. It combines a real local agent workflow with Solana devnet
evidence whenever escrow release or asset movement is claimed.

## What Works Today

The canonical demo proves one bounded product story:

1. Three independent agents register authenticated, key-bound identities.
2. Agent A creates a task and invokes a transaction with Agent B.
3. KAMN records authorization, agreement, escrow, messages, and durable state.
4. Agent A and Agent B receive participant views; Agent C receives a restricted
   public view that can verify shared facts without participant-private fields.
5. Escrow release submits one funded Solana devnet transfer.
6. KAMN retains runtime receipts, projections, websocket evidence, raw Solana
   confirmation, an audit export, and human-readable reports.
7. A standalone verifier rebuilds the evidence chain after all processes exit.

The MVP does **not** prove production readiness, mainnet settlement, production
custody, generalized exchange, arbitrary partition tolerance, or real economic
value. Solana devnet tokens are developer-test tokens.

## Quickstart

Prerequisites are Rust, Bash, Python 3, Pi authenticated with existing Codex
OAuth, three local KAMN identity keys, and funded Solana devnet payer/recipient
keypairs. The exact one-time key and environment setup is in the
[evaluator runbook](docs/validation/mvp-evaluator-demo.md).

Run the canonical evaluator demo after applying that configuration:

```bash
make demo-agent-transaction
```

The command starts the local KAMN stack and three persistent Pi actors, drives
the task and escrow lifecycle, submits one devnet transfer, verifies all three
actor perspectives, and writes the proof bundle. It returns `GO` only after the
standalone verifier passes. Failures return a stable reason and write
`.kamn/demo/latest/NO-GO.txt`.

Verify the report again with no node or agent process running:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo \
  --report .kamn/demo/latest/proof/report.json
```

The immutable run is under `.kamn/demo/<run-id>/`; `latest` is its convenient
alias. Read `.kamn/demo/latest/proof/report.md` for the human report and the
signature-derived Solana Explorer devnet link.

### Local bounded proof

To inspect KAMN's local runtime and proof surfaces without autonomous Pi actors:

```bash
make demo-mvp
```

This is real local execution, but it does not count as settlement or asset
movement unless the generated evidence is explicitly devnet-backed.

## What Each Observer Sees

- **Agent A and Agent B:** shared transaction facts plus their own
  participant-private proof commitments.
- **Agent C:** a restricted public projection containing enough shared facts to
  verify identity, authorization, agreement, receipt order, escrow, and devnet
  settlement without participant-private fields.
- **Evaluator:** the persisted bundle, artifact digests, raw finalized Solana
  response, exact balance deltas, and stable verifier result.

The observers should not see identical data. They should reach the same verdict
from evidence appropriate to their role.

## Claim Boundaries

Every report labels claims explicitly:

| Label | Meaning |
| --- | --- |
| `real` | Local runtime or proof behavior that actually ran. |
| `local-only` | Real local behavior without external value movement. |
| `devnet-backed` | Solana devnet evidence proves the settlement or movement. |
| `dry-run` | Intentional non-live execution; never required-claim success. |
| `placeholder` | Illustrative or unimplemented; never MVP success. |
| `roadmap` | Future work, including production readiness. |

Exchange, escrow settlement, transfer, lamports, asset movement, and value
movement count as success only when labeled `devnet-backed`. Missing or
ambiguous devnet evidence produces `NO-GO`, never a simulated success.

## Repository Map

- `crates/kamn-core`: protocol and domain contracts
- `crates/kamn-node`: local runtime and service API
- `crates/kamn-sdk`: Rust client SDK
- `crates/kamn-agent-lib`: agent identity and authentication helpers
- `crates/kamn-kolme`: live Kolme integration
- `crates/kamn-e2e-harness`: demo, report, and independent verifier
- `specs/`: issue-scoped behavior contracts
- `docs/`: architecture, operations, security, and validation

Start with the [architecture index](docs/architecture/README.md) for system
boundaries and diagrams.

## Development

Run the local quality gates before publishing changes:

```bash
make check
make test
```

The detailed command, policy, and validation inventory lives in the
[README contract reference](docs/developer/readme-contract-reference.md).

## For AI Agents And Maintainers

Read [AGENTS.md](AGENTS.md) before changing the repository. The required flow is
issue first, spec before code, RED/GREEN/REFACTOR, real integration wiring, proof
before completion, and PR review before merge.

Do not weaken tests or claim boundaries. Do not commit secrets, keypairs,
`.kamn/` artifacts, generated package metadata, or unrelated local files.
Prefer consolidating existing surfaces over adding architecture.

## Key Links

- [Evaluator runbook](docs/validation/mvp-evaluator-demo.md)
- [Architecture index](docs/architecture/README.md)
- [README contract reference](docs/developer/readme-contract-reference.md)
- [CI strategy](docs/ci/strategy.md)
- [Kolme devnet operations](docs/planning/kolme-devnet-ops.md)
- [Secure coding](docs/security/secure-coding.md)
