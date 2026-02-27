# Spec: Issue #6130 - FNV-1a ordering fix in deterministic name seed derivation

Status: Accepted
Issue: #6130
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`derive_name_seed_bytes` currently blends bytes via additive mixing before multiplication, which makes the intended FNV-1a step ordering ambiguous and diverges from explicit byte-wise FNV-1a semantics.

## Scope
In scope:
- Implement explicit byte-wise FNV-1a rounds in `derive_name_seed_bytes`.
- Preserve deterministic behavior for a given normalized name.
- Add/adjust regression tests to lock expected deterministic output.

Out of scope:
- Replacing deterministic identity derivation with non-deterministic key provisioning.
- Broad crypto/KDF redesign beyond S-07 ordering correction.

## Acceptance Criteria
- AC-1: Name-seed derivation performs explicit FNV-1a round steps using XOR-then-multiply per processed byte.
- AC-2: Deterministic identity derivation remains stable for identical input and distinct across different names.
- AC-3: Regression/conformance tests lock expected derived signing-key output for a fixed known input.

## Conformance Cases
- C-01 (AC-1): `derive_name_seed_bytes("alice")` path uses explicit FNV-1a round helper and no additive pre-mix shortcut.
- C-02 (AC-2): `from_agent_name("alice")` and `from_agent_name("bob")` derive distinct signing keys, and repeated calls for same input are stable.
- C-03 (AC-3): Known-vector test for `from_agent_name("Alice")` matches updated expected signing key.

## Success Metrics
- `cargo test -p kamn-agent-lib identity -- --nocapture`
- `cargo test -p kamn-agent-lib -- --nocapture`
- `cargo fmt --check`
- `cargo clippy -p kamn-agent-lib --tests -- -D warnings`
