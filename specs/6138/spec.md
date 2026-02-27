# Spec: Issue #6138 - Add Kolme flat JSON parser fuzz target coverage

- Issue: #6138
- Status: Reviewed
- Type: task
- Priority: P2
- Area: qa
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6102

## Problem Statement
R59 S-15 identifies that hand-rolled parser surfaces in Kolme (`split_unquoted` and `parse_flat_json_value_fields`) are high-value fuzz targets. The workspace currently has no dedicated fuzz target for these parser contracts.

## Scope
In scope:
- Add a dedicated cargo-fuzz target that exercises Kolme flat JSON parsing entry points that route through `split_unquoted`.
- Add seed corpus entries and replay metadata for deterministic fuzz replay.
- Update contract docs/tests that inventory required fuzz targets.

Out of scope:
- Parser redesign/refactor.
- New parser API semantics.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: Workspace includes a cargo-fuzz target dedicated to Kolme flat JSON parser surface coverage.
- AC-2: Seed corpus and replay metadata include deterministic entries for the new target.
- AC-3: Contract tests and docs that enumerate required fuzz targets include the new target.

## Conformance Cases
- C-01 (Conformance, AC-1): `fuzz/Cargo.toml` declares `kolme_flat_json_policy_parser` and target source file exists.
- C-02 (Regression, AC-2): replay metadata and corpus include deterministic seeds and replay key for the new target.
- C-03 (Functional/Conformance, AC-3): cargo-fuzz contract tests pass with new target enumerated in docs + inventory checks.

## Success Metrics / Observable Signals
- `cargo test -p kamn-core cargo_fuzz_target_contract` passes with the new target included.
- `cargo fuzz run kolme_flat_json_policy_parser fuzz/corpus/kolme_flat_json_policy_parser -- -runs=1000` executes locally without panics.
