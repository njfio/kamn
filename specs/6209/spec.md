# Spec: Issue 6209 - Correct FNV-1a Ordering in Name-Seed Derivation

- Issue: #6209
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: identity

## Problem Statement

R59 flagged ordering ambiguity in `derive_name_seed_bytes` and noted risk of
silent drift from FNV-1a (XOR-then-multiply) to FNV-1 (multiply-then-XOR).
This issue hardens ordering semantics with explicit helper logic and
regressions.

## Scope

In scope:
1. Make FNV-1a round ordering explicit in the deterministic name-seed path.
2. Add regression coverage that fails if the round order reverts to FNV-1.
3. Preserve deterministic signing-key output compatibility.

Out of scope:
1. Replacing deterministic identity derivation with a cryptographic KDF.
2. Changing normalization rules for agent names.

## Acceptance Criteria

### AC-1 FNV-1a Ordering Explicit
Given the name-seed derivation implementation,
When reviewing the hash round,
Then it uses explicit XOR-then-multiply ordering.

### AC-2 FNV-1 Reversion Guarded
Given representative round inputs,
When comparing FNV-1a and FNV-1 formulas,
Then regression tests assert production logic follows FNV-1a.

### AC-3 Deterministic Output Preserved
Given known name vectors,
When deriving deterministic identity signing keys,
Then existing expected key outputs remain unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6209_name_seed_round_uses_fnv1a_ordering`
- C-02 (AC-2, Unit): `tests::regression_issue_6209_name_seed_round_uses_fnv1a_ordering`
- C-03 (AC-3, Unit): `tests::unit_agent_identity_from_name_builds_expected_did_and_keys`
