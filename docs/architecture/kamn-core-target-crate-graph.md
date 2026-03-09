# KAMN Core Target Crate Graph

## Target Crate Graph (Issue #6647)

kamn_core_target_crate_graph_version=kamn.arch.kamn-core-target-crate-graph.v1
kamn_core_target_crate_graph_status=planned
kamn_core_target_foundational_crates_csv=kamn-types,kamn-crypto,kamn-runtime-guards,kamn-snapshot-journal,kamn-bridges,kamn-data-layer,kamn-kolme,kamn-live-probe-matrix
kamn_core_target_domain_crates_csv=kamn-governance,kamn-escrow,kamn-compliance
kamn_core_target_forbidden_edges_csv=kamn-types->kamn-core,domain-crates->kamn-core-through-shims
kamn_core_target_bridge_rule_csv=kamn-core-reexports-temporary,extracted-crates-must-not-depend-on-kamn-core
kamn_core_target_migration_order_csv=types-inversion,governance,escrow,compliance
kamn_core_target_module_map_source=docs/architecture/kamn-core-module-map.md

This document defines the target layering that should exist after the next
`kamn-core` decomposition waves. It complements the existing tranche roadmap in
`docs/architecture/kamn-core-module-map.md` by specifying allowed dependency
flow, temporary compatibility rules, and the smallest viable `kamn-types`
inversion path.

## Current Graph Summary

- `kamn-core` depends on focused leaf crates that already represent extracted
  capabilities: `kamn-bridges`, `kamn-crypto`, `kamn-data-layer`,
  `kamn-kolme`, `kamn-live-probe-matrix`, `kamn-runtime-guards`, and
  `kamn-snapshot-journal`.
- `kamn-sdk` depends on both `kamn-core` and `kamn-types`.
- `kamn-types` currently depends directly on `kamn-core` because it re-exports
  DID types and parse errors from `kamn-core::did`.
- That means `kamn-types` is currently a facade over `kamn-core`, not a
  foundational crate.

### Concrete Coupling Points

- Manifest edge:
  - `crates/kamn-types/Cargo.toml` declares `kamn-core` in `[dependencies]`.
- Re-export and parse coupling:
  - `crates/kamn-types/src/lib.rs` re-exports `AgentDid`, `KamnDid`,
    `DidDocument`, `DidService`, `DidVerificationMethod`,
    `AgentDidKeyBindingError`, `AgentDidError`, and `KamnDidError` from
    `kamn-core`.
  - the same file constructs `SharedDidParseError` from `AgentDidError` and
    `KamnDidError` and calls `AgentDid::parse(...)` / `KamnDid::parse(...)`
    directly.
- Source ownership today:
  - the concrete DID value types still live in `crates/kamn-core/src/did.rs`.

## Target Layers

### Foundational Layer

These crates may be depended on by `kamn-core`, SDKs, agents, and future domain
crates, but they must not depend on `kamn-core`.

- `kamn-types`
- `kamn-crypto`
- `kamn-runtime-guards`
- `kamn-snapshot-journal`
- `kamn-bridges`
- `kamn-data-layer`
- `kamn-kolme`
- `kamn-live-probe-matrix`

### Domain Layer

These crates should absorb cohesive business domains that are currently still
owned by `kamn-core`.

- `kamn-governance`
- `kamn-escrow`
- `kamn-compliance`

### Orchestration Layer

- `kamn-core` remains the runtime orchestration and composition crate.
- `kamn-node`, `kamn-sdk`, `kamn-agent-lib`, and `kamn-cli` remain consumers of
  the layered workspace graph rather than owners of domain logic.

## Allowed Dependency Directions

- Foundational crates may depend only on std/workspace primitives and other
  foundational crates when there is a clear value-type or helper relationship.
- Domain crates may depend on foundational crates.
- `kamn-core` may depend on foundational crates and domain crates.
- Application crates (`kamn-node`, `kamn-sdk`, `kamn-agent-lib`, `kamn-cli`)
  may depend on `kamn-core` plus foundational crates when they need stable leaf
  types directly.
- Domain crates must not depend back on `kamn-core` through compatibility shims.
- `kamn-types` must not depend on `kamn-core` in the target state.

## Temporary Bridge Rules

- `kamn-core` may re-export APIs from extracted crates temporarily to preserve
  compatibility while callers migrate.
- Extracted crates must not re-import those APIs through `kamn-core`.
- New shared value types should be introduced in `kamn-types` first, not added
  to `kamn-core` and mirrored later.
- Shim retirement should happen only after downstream imports are moved to the
  extracted crate paths.

## kamn-types Inversion Plan

### Current Coupling

`kamn-types` currently depends on `kamn-core` because it forwards and uses the
DID value surface from `kamn-core::did`.

Current directly coupled surface:
- `AgentDid`
- `KamnDid`
- `AgentDidError`
- `KamnDidError`
- `AgentDidKeyBindingError`
- `AgentDidMetadata`
- `DidDocument`
- `DidService`
- `DidVerificationMethod`

### First Inversion Wave

Move the DID value types and parse errors out of `kamn-core::did` into
`kamn-types`, keeping runtime registry logic in `kamn-core`.

First-wave owned surface in `kamn-types`:
- `AgentDid`, `KamnDid`
- `DidDocument`, `DidService`, `DidVerificationMethod`
- `AgentDidError`, `KamnDidError`, `AgentDidKeyBindingError`
- `AgentDidMetadata`
- Canonical parse helpers now exposed from `kamn_types::did`

What stays in `kamn-core` initially:
- `did_registry`
- runtime handshake and orchestration flows
- higher-level identity workflows that compose runtime state and persistence

## Candidate Module Mapping

### Target: kamn-types

- First wave modules/value surface:
  - `did`, `agent_key_hierarchy`, `key_lifecycle`, `key_recovery`
- Scope note:
  - only the reusable value-type and parse boundary moves first; registry and
    orchestration logic stay in `kamn-core` until later follow-up issues.

### Target: kamn-governance

- Candidate modules:
  - `governance_workflow`, `operator_actions`, `operator_dashboard_api`, `operator_dashboard_ui`
- Scope note:
  - these modules already form a policy and operator control-plane boundary and
    should stop accreting inside `kamn-core`.

### Target: kamn-escrow

- Candidate modules:
  - `task_operations`, `task_lifecycle`, `task_payment`, `task_artifacts`, `escrow`, `service_marketplace`, `token`
- Scope note:
  - task DAG progression and settlement state belong in one economic domain
    slice instead of remaining distributed through the core orchestration crate.

### Target: kamn-compliance

- Candidate modules:
  - `content_storage`, `content_retrieval`, `content_lifecycle`, `content_replication`, `data_classification`, `redaction_compliance`, `audit_exports`
- Scope note:
  - content policy, retention, replication, and evidence export should become a
    dedicated compliance/content policy crate.

### Retained In kamn-core During These Waves

- Runtime orchestration and state composition:
  - `runtime`, `bootstrap`, `config`, `migrations`, `state`, `durable_guard_store`
- Messaging and channel control plane:
  - `message_envelope`, `message_lifecycle`, `channel_models`, `channel_policies`, `instruction_verify`
- Transport/runtime integration:
  - `p2p_transport`, `kolme_runtime_commit`, `runtime_*`

## Migration Order

1. `types-inversion`
- move the first-wave DID/value surface into `kamn-types`
- invert `kamn-core` to depend on `kamn-types`
- keep `kamn-core` re-exports as temporary compatibility shims

2. `governance`
- extract the governance/operator control plane into `kamn-governance`
- keep orchestration entrypoints in `kamn-core`

3. `escrow`
- extract task/escrow/economic modules into `kamn-escrow`
- preserve runtime composition from `kamn-core`

4. `compliance`
- extract content/compliance/audit policy modules into `kamn-compliance`
- keep remaining runtime composition and cross-domain orchestration in `kamn-core`

## Graph Check Plan

Follow-up extraction issues should add or extend graph-check contracts that
assert:
- `kamn-types` has no `kamn-core` dependency
- new domain crates do not depend back on `kamn-core`
- `kamn-core` is allowed to depend on extracted crates during the migration
  window
