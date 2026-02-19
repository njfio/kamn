# Issue #4140 Plan

- Issue: #4140
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Approach
1. Add RED tests in `cargo_fuzz_target_contract.rs` for deterministic seed provenance markers in metadata and CI docs.
2. Add provenance marker fields to `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`.
3. Add corresponding provenance contract markers to `docs/ci/strategy.md`.
4. Run targeted contract/doc tests and strict formatting/lint checks.

## Affected Files
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`
- `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`
- `docs/ci/strategy.md`
- `specs/4140/spec.md`
- `specs/4140/plan.md`
- `specs/4140/tasks.md`

## Risks and Mitigations
- Risk: marker assertions become brittle with doc wording changes.
  - Mitigation: assert stable marker keys/values rather than narrative prose.
- Risk: docs-contract changes trigger unrelated CI doc tests.
  - Mitigation: run `ci_strategy_docs` locally before PR.
- Risk: shell-surface growth.
  - Mitigation: avoid adding shell wrappers/scripts; use metadata/docs/test updates only.

## Interface Contract
- Contract-test and docs metadata scope only.
- No API or runtime behavioral interface changes.

## ADR
- Not required (bounded governance contract update).
