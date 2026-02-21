# Issue #5505 Plan - Spec-Volume Remediation Plan Contractization

## Approach
1. Add remediation tranche-plan markers to `gaps-and-issues-r50.md`.
2. Add a dedicated docs-contract test validating arithmetic and status markers.
3. Run targeted test and format checks.

## Affected Modules
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r50_spec_volume_remediation_docs_contract.rs`
- `specs/milestones/r50-18-spec-volume-guardrail-remediation-contracts/index.md`
- `specs/5505/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: remediation marker arithmetic drift.
  - Mitigation: numeric parsing + formula assertions in tests.

## Interfaces / Contracts
- Docs-contract markers only.

## Validation Strategy
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r50_governance_loop_mitigation_docs_contract -- --nocapture`
- `TMPDIR=/home/n/Code/kamn-r51/.tmp-cargo cargo fmt --check`
