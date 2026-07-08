# KAMN

KAMN (Kolme AI Agent Messaging Network) is a privacy-first, auditable coordination layer for autonomous agents. It is meant to let agents identify themselves, exchange signed work messages, leave durable proof trails, and make settlement claims only when there is evidence behind them.

This repository is the Rust implementation and validation workspace for that system. It contains the local runtime, service API, SDK and agent helpers, proof/report harnesses, live-provider integration surfaces, and process tooling used to evolve KAMN without overstating what is real.

The current MVP is not a production network. It is a locally runnable, evaluator-friendly demo that proves one coherent product story: local authenticated agent flow plus optional Solana devnet-backed settlement evidence when funded devnet keypairs are configured.

## What KAMN Proves Today <!-- ## What This Repository Contains -->

The repository currently contains proof surfaces for:

- A local KAMN service API/runtime starts and produces proof artifacts.
- Alice/Bob agent identities are authenticated in local proof output.
- A signed message or task flow is recorded.
- Durable state is written under the demo run directory.
- Relay/projection state and websocket event visibility are captured.
- An audit/proof export and human-readable proof report are generated.
- Settlement or asset-movement success is claimed only as `devnet-backed`, using Solana devnet transaction, balance, and persisted KAMN state evidence.

The demo does not prove production readiness, mainnet settlement, generalized exchange, broad bridge finality, arbitrary partition tolerance, or real economic value. Solana devnet tokens are developer-test tokens.

## MVP Demo Quickstart <!-- ## Quickstart -->

Prerequisites:

- Rust toolchain (`cargo`, `rustc`)
- Bash
- Python 3
- Solana CLI only for independently confirming devnet transactions

Run the local-only MVP demo:

```bash
make demo-mvp
```

Verify the generated proof report:

```bash
cargo run -p kamn-e2e-harness -- verify-mvp-demo --report .kamn/demo/latest/proof/report.json
```

Expected top-level artifacts:

```text
.kamn/demo/latest/proof/report.json
.kamn/demo/latest/proof/report.md
```

The report links to the concrete run directory under `.kamn/demo/<run-id>/`, including local signed-flow, service API, websocket, audit, and settlement proof files where applicable.

For the funded Solana devnet-backed path:

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

If the devnet path is fully configured and funded, the report can return `GO` with a `devnet_settlement_asset_movement` claim labelled `devnet-backed`. If devnet evidence is unavailable, the honest result is `NO-GO`, not a local-only settlement pass.

Detailed evaluator runbook:

- `docs/validation/mvp-evaluator-demo.md`

## Claim Boundaries

KAMN proof reports use explicit claim labels:

| Label | Meaning |
| --- | --- |
| `real` | Local runtime or proof behavior that actually ran. |
| `local-only` | Real local behavior without external value movement. |
| `devnet-backed` | Solana devnet-backed evidence exists for the settlement or asset-movement claim. |
| `dry-run` | Intentional non-live execution that must not count as MVP success for required claims. |
| `placeholder` | Unimplemented or illustrative output that must not count as MVP success. |
| `roadmap` | Future work or non-MVP scope, including production readiness. |

Required MVP success claims cannot be `dry-run` or `placeholder`.

Any claim involving exchange, escrow, settlement, transfer, lamports, asset movement, or value movement must be `devnet-backed`. KAMN must not turn fake in-memory movement into a settlement success claim.

## Repository Map <!-- ## Architecture Map -->

- `crates/kamn-core`: protocol/domain logic and contract suites
- `crates/kamn-node`: node runtime entrypoint and service API
- `crates/kamn-sdk`: Rust SDK client surface
- `crates/kamn-agent-lib`: agent-facing auth/identity helpers
- `crates/kamn-kolme`: Kolme live provider integration layer
- `crates/kamn-e2e-harness`: MVP demo, report verifier, and end-to-end proof harnesses
- `scripts/`: CI, contract lanes, and deterministic validation utilities
- `docs/`: architecture, operations, security, validation, and planning references
- `specs/`: issue-scoped specs that precede implementation work

## Validation Lanes

Fast local gates:

```bash
make check
make test
make ci-tools
```

Core validation commands:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test
```

Evaluator and live-network lanes:

```bash
make demo-mvp
make smoke-live-network
make deep-live-network
make demo-localhost-transport
```

Live HTTPS dependency posture checks for `kamn-core`:

```bash
cargo check -p kamn-core --features live-https
cargo check -p kamn-core --no-default-features
```

## For AI Agents And Maintainers <!-- ## Workflow -->

Start with `AGENTS.md`. The repository process is issue-first, spec-before-code, TDD, integration-wired, and proof-before-completion.

Required change flow:

1. Open or reuse a GitHub issue with problem statement, acceptance criteria, and non-goals.
2. Write `specs/<issue>-<slug>.md` before implementation or test changes.
3. Add red tests derived from the spec.
4. Implement the smallest green path.
5. Refactor deliberately; do not skip the refactor phase.
6. Wire the behavior into real entrypoints; no floating code.
7. Run focused tests, local gates, proof commands, and PR checks before merging.

Agent guardrails:

- Do not weaken tests, lint, clippy, formatting, proof semantics, or claim boundaries.
- Do not claim exchange, escrow, settlement, or asset movement unless the evidence is devnet-backed.
- Do not commit secrets, devnet keypairs, `.kamn/` proof artifacts, generated package metadata, or unrelated local files.
- Prefer consolidating existing working surfaces over adding architecture.
- Keep root README human-first; put exhaustive operational detail in linked docs.

## Key Links

Start here for system navigation:

- `docs/architecture/README.md`

Core architecture references:

- `docs/architecture/runtime-layout.md`
- `docs/architecture/service-runtime.md`
- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/kamn-node-module-map.md`
- `docs/foundation/kolme-runtime-architecture.md`
- `docs/foundation/runtime-network.md`
- `docs/architecture/adr-kamn-core-live-tls-transport.md`

MVP and validation references:

- `docs/validation/mvp-evaluator-demo.md`
- `docs/developer/readme-contract-reference.md`
- `docs/ci/strategy.md`
- `docs/planning/engineering-hardening-wave.md`
- `docs/planning/live-network-wave.md`
- `docs/planning/kolme-devnet-ops.md`
- `docs/developer/rustdoc-publishing.md`
- `docs/security/secure-coding.md`
- `docs/security/tls-hardening.md`

## Contract Reference

Detailed command matrices, contract markers, policy snippets, and lane-specific references are maintained in:

- `docs/developer/readme-contract-reference.md`

This keeps the root README useful as an onboarding front door while preserving deterministic contract markers in a stable docs location.
