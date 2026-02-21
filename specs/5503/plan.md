# Issue #5503 Plan - Governance-Loop Mitigation Enforcement

## Approach
1. Add governance-loop mitigation marker section to `gaps-and-issues-r50.md`.
2. Add arithmetic remediation markers for spec-volume reduction targets.
3. Add new docs-contract test file to enforce marker presence and consistency.
4. Run targeted test and formatting gates.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_governance_loop_mitigation_docs_contract.rs`
- `specs/milestones/r50-17-governance-loop-mitigation-contracts/index.md`
- `specs/5503/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: arithmetic marker drift.
  - Mitigation: parse markers in tests and verify formulas.

## Interfaces / Contracts
- Docs-contract markers only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_governance_loop_mitigation_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
