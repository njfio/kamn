# Spec: Issue #6064 - Gate deterministic name-derived identity outside production

- Issue: #6064
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6062

## Problem Statement
`AgentIdentity::from_agent_name` deterministically derives a secp256k1 private key from a public agent name. Without an explicit production guard, this can be used in production with fully predictable keys.

## Scope
In scope:
- Add explicit production-mode gate logic for deterministic identity derivation.
- Preserve debug/test ergonomics by allowing deterministic derivation in non-production contexts.
- Add clear API security warning for `from_agent_name`.
- Add tests for gate decision logic and error behavior.

Out of scope:
- Replacing deterministic derivation algorithm.
- DID registry/on-chain key binding.
- Key management UX redesign.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Production-mode decision logic rejects deterministic identity derivation by default.
- AC-2: Non-production (debug) decision logic continues to allow deterministic identity derivation.
- AC-3: Explicit override enables deterministic derivation in production-mode decision logic.
- AC-4: `from_agent_name` API has a clear security warning describing non-production intent.

## Conformance Cases
- C-01 (Unit, AC-1): policy helper returns deny for `(debug=false, env missing)`.
- C-02 (Unit, AC-2): policy helper returns allow for `(debug=true, env missing)`.
- C-03 (Unit, AC-3): policy helper returns allow for `(debug=false, env=true|1|yes)`.
- C-04 (Functional, AC-4): deterministic identity call returns explicit policy error message when blocked.

## Success Metrics / Observable Signals
- RED test fails without production gating helper and blocked-error path.
- GREEN tests pass for helper decision matrix and blocked-call behavior.
- `cargo test -p kamn-agent-lib identity` passes.
