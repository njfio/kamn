# Plan: Issue #6138

## Approach
1. Add `kamn-kolme` path dependency to the fuzz crate and declare a new fuzz binary target.
2. Implement `fuzz/fuzz_targets/kolme_flat_json_policy_parser.rs` to feed bounded UTF-8 payloads into `parse_flat_json_value_fields` and related field extractors.
3. Add deterministic seed corpus files and replay metadata entry for the new target.
4. Update parser-fuzz documentation markers and cargo-fuzz contract tests to require the new target.
5. Run targeted contract tests and a bounded fuzz smoke run.

## Affected Modules
- `fuzz/Cargo.toml`
- `fuzz/fuzz_targets/kolme_flat_json_policy_parser.rs`
- `fuzz/corpus/kolme_flat_json_policy_parser/*`
- `fuzz/corpus/replay-metadata/cargo-fuzz-seed-corpus-v1.json`
- `docs/ci/strategy.md`
- `docs/architecture/parser-protocol-assurance.md`
- `docs/security/secure-coding.md`
- `crates/kamn-core/tests/cargo_fuzz_target_contract.rs`

## Risks / Mitigations
- Risk: fuzz target drifts from documented inventory.
  Mitigation: extend contract tests/docs markers in same patch.
- Risk: unbounded input allocations.
  Mitigation: cap fuzz input length before parser invocation.

## Interfaces / Contracts
- No production runtime API behavior changes.
- Cargo fuzz target inventory contract expands by one explicit target.
