---
date: 2026-06-26
topic: kamn-forward-strategy
artifact_status: historical
current_status: superseded
---

# KAMN Forward Strategy

> Historical decision artifact. This dated branch analysis is superseded by
> the completed agent-transaction plan. It is not production or mainnet
> evidence; current settlement claims remain limited to Solana devnet.

## Problem Frame
KAMN is no longer just paper architecture. Current `main` contains callable Rust
runtime, CLI, MCP, SDK, service API, live proof, and E2E harness surfaces. The
project's own validation docs now list multiple bounded proof slices, including
service API, TCP relay, restart persistence, escrow, bridge, live Solana devnet,
websocket, MCP, CLI, and asset-movement lanes.

The project is still not ready to be described as broadly production-ready. The
dominant risk is proof dilution: too many specs, scripts, governance contracts,
and validation surfaces relative to the number of undeniable product/runtime
paths a maintainer or evaluator can run and understand.

Branch scan result: after `git fetch --all --prune`, `origin/main` at
`560cbbb3` is the latest integrated product state. Newer open PR branches are
Dependabot-only Cargo dependency bumps. The freshest non-dependency unmerged
feature branch found, `origin/6880-verify-task-operations`, is stale relative to
`main` and should not replace `main` as the analysis base.

## What Works
- Runtime entrypoints are real: `kamn-node`, `kamn-cli`, `kamn-mcp-server`, and
  `kamn-e2e-harness` are callable binaries in the workspace.
- Service API has a substantial route surface, auth taxonomy, replay guard,
  anti-spam posture, websocket fanout, relay spool, and state persistence paths.
- Persistence is better than old file-only claims: JSON state writes are atomic,
  SQLite state-file storage is supported by extension, and relay projection is
  idempotent.
- Current validation docs are unusually honest about proof boundaries; the
  canonical `current-proven-runtime-slices.md` index separates proven slices
  from unproven production, consensus, finality, and fault-tolerance claims.
- E2E live CI exists and can run SDK-direct and MCP-agent modes against a KAMN
  stack plus a checked-out Kolme local API runtime.

## What Does Not Work Yet
- Local quality gates are not green on current `main`: `cargo fmt --check`
  reports formatting drift and strict clippy fails in `kamn-core` when warnings
  are promoted to errors.
- `cargo check --workspace --all-targets --all-features` passes, but with many
  warnings. That is not enough for the repo's own `make check` contract.
- Broad production readiness, consensus/multi-node finality, generalized
  external settlement, broad live economic-settlement parity, and global fault
  tolerance remain explicitly unproven.
- The E2E harness still contains deterministic infrastructure placeholders for
  parts of infra/deploy/evidence/teardown modeling, so harness pass semantics
  need careful labeling.
- CI is layered and sophisticated, but proof is fragmented across fast gate,
  deep validate, live workflows, dry-run policy checks, and local-heavy opt-ins.

## What Is Real
- `kamn-node` dispatches CLI parsing into runtime orchestration and serves
  service API / observability endpoints when configured.
- Service API message send persists message state, appends recipient relay spool
  entries, and publishes websocket lifecycle events.
- Service API state defaults to a temp-file path derived from bind address, can
  be overridden, and can use SQLite when the state-file extension is `.sqlite`,
  `.sqlite3`, or `.db`.
- Websocket support is no longer a one-shot-only implementation; current source
  has broadcast fanout, sequence numbers, state-transition mode, and presence
  mode.
- Kolme-related surfaces include real cryptographic signing paths and live
  provider integration claims in source/docs, not only markdown plans.

## What Is Not Real Enough
- The main user/product story is still hard to evaluate from one command and one
  operator narrative. A newcomer sees a strong platform, but also a very large
  governance system.
- Production datastore posture is not yet an obvious default path. File/SQLite
  state-file support proves durability slices, but the product still needs a
  crisp production storage story.
- Some broad claims are only partially supported by bounded slices. The project
  should keep marketing, README, and roadmap language anchored to validated
  proof slices.
- Security and supply-chain posture have advisory/non-blocking areas and should
  be separated from hard merge gates in status language.

## Requirements
- R1. Establish a single "what KAMN does today" operator path that can be run
  from fresh checkout and produces one human-readable proof bundle.
- R2. Make local quality gates green before feature expansion: formatting,
  strict clippy, and targeted tests should match the README and `Makefile`
  contract.
- R3. Convert current bounded proof slices into a claim matrix: claim, evidence
  command, proof status, non-goals, and freshness date.
- R4. Prioritize one end-to-end product path over new architecture breadth:
  authenticated agent message or task flow, durable state, relay/projection,
  websocket visibility, and audit/proof export.
- R5. Label harness placeholders and dry-run lanes explicitly in generated
  evidence so they cannot be mistaken for live runtime proof.
- R6. Reduce proof fragmentation by defining one recommended local smoke lane,
  one CI PR lane, and one scheduled/live lane for evaluator confidence.
- R7. Stop adding new major nouns until each existing major noun has equivalent
  runtime depth or is demoted to roadmap language.

## Success Criteria
- A new maintainer can answer "what does KAMN do?" by running one smoke command
  and reading one proof report.
- `make check` passes locally on the selected branch.
- The claim matrix has no broad production claim without a linked command,
  validation doc, and explicit non-goal.
- The next feature PR improves one runtime proof path instead of adding another
  isolated spec/governance layer.
- Dry-run, placeholder, local-heavy, live, and production terms are used
  consistently in docs and evidence output.

## Scope Boundaries
- Do not redesign the whole architecture before cleaning proof and local gate
  health.
- Do not weaken tests, clippy, formatting, or governance gates to make the repo
  look green.
- Do not market broad production readiness until consensus/finality/fault
  tolerance and generalized settlement claims have matching live evidence.
- Do not start with SDK expansion or new surfaces; finish the strongest current
  path first.

## Key Decisions
- Base analysis on `origin/main`, not a newer Dependabot branch: dependency
  bumps do not represent current product strategy.
- Treat the project as partially real, not fake: current code has callable
  runtime and proof paths, but production maturity remains bounded.
- Move forward by consolidation and proof depth, not by creating more specs,
  scripts, or validation taxonomies.
- First engineering objective should be local quality gate recovery because it
  blocks trustworthy iteration.

## Dependencies / Assumptions
- GitHub branch and PR metadata are current as of the fetch performed for this
  brainstorm.
- Live E2E claims depend on external Kolme checkout/build availability and
  secrets for MCP-agent mode where required.
- This brainstorm did not execute live E2E workflows or long full test suites.

## Outstanding Questions

### Resolve Before Planning
- None.

### Deferred to Planning
- [Affects R1][Technical] Which existing smoke/live lane should become the
  canonical one-command proof path?
- [Affects R2][Technical] Should strict clippy be fixed in one gate-recovery
  issue or split by crate/module to preserve small TDD arcs?
- [Affects R4][Product] Is the primary product proof path agent messaging,
  task lifecycle, escrow settlement, or Solana-backed asset movement?
- [Affects R6][Technical] Which CI lanes should be required for PR confidence
  versus scheduled confidence?

## Next Steps
-> /prompts:ce-plan for structured implementation planning.
