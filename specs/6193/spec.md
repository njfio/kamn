# Spec: Issue 6193 - Signer Adapter Must Not Clone Private Key Material

- Issue: #6193
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: security

## Problem Statement

`KolmeForkSecp256k1SignerAdapter` previously derived `Clone`, allowing implicit duplication
of private-key-bearing signer state.

## Scope

In scope:
1. Remove `Clone` derivation from `KolmeForkSecp256k1SignerAdapter`.
2. Add regression guardrails preventing clone derive reintroduction.
3. Validate signer/runtime test lanes remain green.

Out of scope:
1. Re-architecting signer transport APIs.
2. Introducing shared ownership wrappers for signer state.

## Acceptance Criteria

### AC-1 Clone Removal
Given signer adapter source,
When `KolmeForkSecp256k1SignerAdapter` is defined,
Then it does not derive `Clone`.

### AC-2 Regression Guard
Given boundary contract tests,
When signer adapter source changes,
Then tests fail if `Clone` derive is reintroduced.

### AC-3 Runtime Compatibility
Given current signer workflows,
When the clone derive is removed,
Then signer tests continue to pass.

## Conformance Cases

- C-01 (AC-1, Unit): `crates/kamn-node/src/signer/signer_adapter.rs` no longer declares `#[derive(Debug, Clone)]`.
- C-02 (AC-2, Contract): `signer_adapter_boundary_contract::source_enforces_signer_adapter_ownership_without_reinline_backslide`.
- C-03 (AC-3, Integration): `main_tests::signer_tests::*` signer lanes remain green.
