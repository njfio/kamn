# Issue 6226 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
`AgentDid` currently exists as separate implementations in `kamn-core` and `kamn-sdk`, which fragments validation semantics and type identity across crates. This increases drift risk and creates avoidable adapter overhead.

## Scope
In scope:
- Introduce a shared `kamn-types` crate as the common type surface for canonical DID primitives.
- Export canonical `AgentDid` (and related DID errors/types needed by SDK adoption) through `kamn-types`.
- Migrate `kamn-sdk` to consume shared `AgentDid` from `kamn-types`.
- Add compatibility conversion from shared `AgentDidError` to `SdkError` so existing `?` call sites remain fail-closed.

Out of scope:
- Full extraction/refactor of all DID logic from `kamn-core`.
- Broad migration of all crates beyond `kamn-sdk` for this issue.

## Acceptance Criteria
- AC-1: Workspace includes new `crates/kamn-types` crate and registers it in workspace members.
- AC-2: `kamn-types` exposes canonical `AgentDid` surface shared across crates.
- AC-3: `kamn-sdk` no longer defines a local `AgentDid` struct and instead uses `kamn-types::AgentDid`.
- AC-4: `kamn-sdk` compiles with shared DID error mapping via `From<AgentDidError> for SdkError` (or equivalent fail-closed mapping).
- AC-5: Targeted verification passes for `kamn-types`, `kamn-sdk`, and `kamn-agent-lib`.

## Conformance Cases
- C-01 (AC-1, Conformance): root `Cargo.toml` includes `crates/kamn-types` in workspace members.
- C-02 (AC-2, Unit/Conformance): `kamn-types` exports `AgentDid` and parse API usable from dependent crates.
- C-03 (AC-3, Unit/Conformance): `kamn-sdk/src/types.rs` contains no local `AgentDid` definition and compiles using shared `AgentDid` import.
- C-04 (AC-4, Unit/Conformance): invalid shared DID parse in SDK context maps to deterministic `SdkError::InvalidInput`.
- C-05 (AC-5, Functional): `cargo test -p kamn-types` passes.
- C-06 (AC-5, Functional): `cargo test -p kamn-sdk` passes.
- C-07 (AC-5, Functional): `cargo test -p kamn-agent-lib` passes.

## Success Metrics
- Single canonical `AgentDid` type surface is consumed by `kamn-sdk`.
- No behavior regression in SDK/agent-lib test suites for DID parsing call paths.
