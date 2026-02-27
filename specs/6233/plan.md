# Plan: Issue #6233 - Migrate README Contract Lane from Shell to Rust Integration Test

## Approach

1. Add a RED Rust test asserting the shell wrapper is thin and README contract validation is Rust-owned.
2. Add fixture `.ci/readme_contract_required_snippets.txt` containing required marker snippets.
3. Implement `crates/kamn-core/tests/readme_contract_lane.rs` to read fixture and validate README docs.
4. Rewrite `scripts/ci/test_readme_contract.sh` as a compatibility wrapper that invokes the Rust lane.
5. Run targeted test/doc lanes and format checks.

## Affected Modules

- `scripts/ci/test_readme_contract.sh`
- `.ci/readme_contract_required_snippets.txt` (new)
- `crates/kamn-core/tests/readme_contract_lane.rs` (new)

## Risks and Mitigations

- Risk: drift between prior shell marker set and new Rust fixture.
  - Mitigation: bootstrap fixture directly from current script marker list and validate non-empty inventory.
- Risk: CI callsites depend on shell script output text.
  - Mitigation: keep wrapper command path and success message stable.

## Verification

- `cargo fmt --all --check`
- `cargo test -p kamn-core --test readme_contract_lane -- --nocapture`
- `bash scripts/ci/test_readme_contract.sh`
- `cargo test -p kamn-core --test readme_compact_contract -- --nocapture`
