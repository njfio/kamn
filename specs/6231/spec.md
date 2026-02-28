# Spec: Issue #6231 - Begin kamn-core Extraction Wave 1 (Crypto, Bridges, Data-Layer)

- Status: Implemented
- Priority: P1
- Parent: #6223
- Milestone: R59 Swarm Gap Closure

## Problem Statement

`kamn-core` remains a large multi-domain crate with weak crate boundaries. We need a concrete extraction wave that moves real domain modules into focused crates while preserving compatibility for existing `kamn-core` consumers.

## Scope

In scope:
- Introduce focused crates for extraction wave 1:
  - `kamn-crypto`
  - `kamn-bridges`
  - `kamn-data-layer`
- Move one concrete module into each new crate:
  - crypto: direct-message crypto module
  - bridges: cross-chain receipt normalization module
  - data-layer: shared SHA-256 hashing helper module
- Keep `kamn-core` API compatibility using façade re-exports for moved modules.
- Preserve compile/test behavior for moved surfaces.

Out of scope:
- Full decomposition of all bridge/data-layer/crypto modules.
- Breaking API changes for existing callers.

## Acceptance Criteria

### AC-1 New Focused Crate Boundaries
Given extraction wave 1,
When workspace members are evaluated,
Then focused crates `kamn-crypto`, `kamn-bridges`, and `kamn-data-layer` exist and are registered.

### AC-2 Compatibility Facades in kamn-core
Given existing `kamn-core` import paths,
When callers compile against `kamn-core`,
Then moved modules remain available through `kamn-core` façade modules with no path breakage.

### AC-3 Compile/Test Parity for Moved Modules
Given moved module behavior,
When crate-level tests execute,
Then moved module tests pass in extracted crates and `kamn-core` compiles against new dependencies.

## Conformance Cases

- C-01 (AC-1, Unit): workspace/members and extracted crate manifests are present and valid.
- C-02 (AC-2, Integration): façade module contract test verifies `kamn-core` re-export wiring for moved modules.
- C-03 (AC-3, Regression): extracted crate tests and targeted `kamn-core` build/test checks pass.
