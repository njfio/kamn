# Plan — #4397

Status: Reviewed

## Approach

- Introduce deterministic persistence gate constants in `validate_persistence_adapters_live.sh`.
- Emit marker fields in both stdout and JSON report payload.
- Enforce deterministic mismatch detection in test tamper paths.
- Update persistence governance documentation and docs-contract tests.

## Affected Areas

- `scripts/runtime/validate_persistence_adapters_live.sh`
- `scripts/runtime/test_validate_persistence_adapters_live.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations

- Risk: mismatch between docs and script marker names.
  - Mitigation: add docs-contract assertions in same change.

## Validation

- Targeted script tests and docs tests, then fmt/clippy/test gates.
