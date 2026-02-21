# Issue #5501 Plan - Feature Provenance Marker Reconciliation

## Approach
1. Add deterministic feature provenance markers and status highlight line to `gaps-and-issues-r49.md`.
2. Extend `review_r49_docs_contract` assertions and integration consistency checks.
3. Run targeted docs-contract tests and format check.

## Affected Modules
- `docs/review/gaps-and-issues-r49.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `specs/milestones/r50-16-r49-production-feature-provenance-parity-refresh/index.md`
- `specs/5501/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker mismatch between doc and test.
  - Mitigation: parse marker values in integration test and assert expected numeric constants.

## Interfaces / Contracts
- Docs-contract markers only; no runtime/public API changes.

## Validation Strategy
- `cargo test -p kamn-core --test review_r49_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn/.tmp-cargo cargo fmt --check`
