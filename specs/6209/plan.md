# Plan: Issue 6209 - Correct FNV-1a Ordering in Name-Seed Derivation

- Issue: #6209
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. In `crates/kamn-agent-lib/src/identity.rs`, extract the 64-bit hash round
   into a dedicated helper implementing FNV-1a ordering.
2. Refactor `derive_name_seed_bytes` to use that helper without changing output
   semantics.
3. Add regression test coverage that distinguishes FNV-1a from FNV-1 ordering.
4. Run scoped formatting, lint, and tests for `kamn-agent-lib`.

## Affected Modules

- `crates/kamn-agent-lib/src/identity.rs`

## Risks and Mitigations

1. Risk: accidental deterministic output drift can break test vectors.
   - Mitigation: keep existing expected signing key test as compatibility guard.
2. Risk: future edits could inline and reorder hash rounds again.
   - Mitigation: explicit helper plus regression naming tied to issue id.
