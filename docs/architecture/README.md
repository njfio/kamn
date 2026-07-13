# Architecture Navigation Index

schema_version=kamn.docs.architecture-navigation-index.v1
diagram_catalog_status=active
index_last_reviewed=2026-07-13

This index is the canonical navigation entrypoint for architecture artifacts and
diagram references.

## MVP Surface Classes

- **canonical runtime:** `make demo-agent-transaction` drives three Pi actors
  through the local service API and requires finalized Solana devnet settlement
  evidence before `GO`.
- **compatibility:** `make demo-mvp`, SDK/CLI compatibility lanes, and generic
  harness commands remain supported but do not replace the canonical story.
- **local-only:** local runtime, identity, task, persistence, relay, websocket,
  and audit proof are real without claiming external value movement.
- **dry-run:** planning and policy lanes execute no live settlement and cannot
  contribute claims to canonical `GO`.
- **placeholder:** generic run-contract orchestration markers are illustrative,
  non-authoritative, and disconnected from `demo-agent-transaction` success.
- **roadmap:** production readiness, mainnet custody, multi-authority release,
  disputes, and generalized exchange are not claimed.

## Test Taxonomy

- **behavior:** deterministic domain rules, authorization, state transitions,
  idempotency, and verifier rejection contracts.
- **integration:** real in-process or process-boundary wiring across KAMN
  components, persistence, SDK, MCP, CLI, and proof surfaces.
- **live:** explicitly configured external runtime or devnet execution with
  fail-closed evidence requirements.
- **docs-contract:** navigation, command, schema, and policy text markers; these
  do not substitute for runtime behavior proof.
- **legacy compatibility:** retained old-format and old-command boundaries that
  cannot contribute to canonical MVP success.

## Module Maps

- `docs/architecture/kamn-core-module-map.md`
- `docs/architecture/kamn-core-module-map.md#decomposition-tranche-roadmap-issue-6275`
- `docs/architecture/kamn-core-target-crate-graph.md`
- `docs/architecture/kamn-node-module-map.md`

## Crate Architecture Notes

- `docs/architecture/kamn-agent-lib.md`
- `docs/architecture/kamn-cli.md`
- `docs/architecture/kamn-crypto.md`
- `docs/architecture/kamn-data-layer.md`
- `docs/architecture/kamn-e2e-harness.md`
- `docs/architecture/kamn-governance.md`
- `docs/architecture/kamn-snapshot-journal.md`
- `docs/architecture/kamn-types.md`

## Runtime And Service Flows

- `docs/architecture/runtime-layout.md`
- `docs/architecture/service-runtime.md`
- `docs/architecture/data-layer-runtime-wiring.md`
- `docs/architecture/block-pipeline.md`
- `docs/architecture/p2p-transport.md`
- `docs/architecture/kolme-live-integration.md`
- `docs/architecture/kolme-runtime-commit.md`
- `docs/architecture/signer-lifecycle.md`
- `docs/architecture/persistence-backends.md`
- `docs/architecture/did-chain-adapter.md`
- `docs/architecture/parser-protocol-assurance.md`
- `docs/architecture/service-api-delivery-flow.md`

## Reliability Contracts

- `docs/architecture/helper-canonicalization.md`

## Decision Records

- `docs/architecture/adr-kamn-core-live-tls-transport.md`
- `docs/architecture/adr-kamn-sdk-service-https-transport.md`
- `docs/architecture/adr-001-production-message-crypto-primitives.md`
- `docs/architecture/adr-002-runtime-guards-phase1-extraction.md`
- `docs/architecture/adr-003-kamn-core-wave2-shim-retirement.md`
- `docs/architecture/adr-cargo-audit-ci-gate.md`
- `docs/architecture/adr-critical-path-assurance-gates.md`

## Diagram Catalog

- `diagram:runtime-layout` => `docs/architecture/runtime-layout.md`
- `diagram:service-runtime` => `docs/architecture/service-runtime.md`
- `diagram:block-pipeline` => `docs/architecture/block-pipeline.md`
- `diagram:p2p-transport` => `docs/architecture/p2p-transport.md`
- `diagram:kolme-live-integration` => `docs/architecture/kolme-live-integration.md`
- `diagram:signer-lifecycle` => `docs/architecture/signer-lifecycle.md`
