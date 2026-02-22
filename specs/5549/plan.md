# Issue #5549 Plan - Review Artifact Snapshot Semantics and Reconciliation-Loop Guardrails

## Approach
1. Add RED docs-contract test coverage for R50+ snapshot-semantics markers and reconciliation-chain limits.
2. Update review marker policy documentation in `docs/review/README.md`.
3. Update `docs/review/gaps-and-issues-r50.md` with deterministic snapshot-semantics markers.
4. Relax `review_r49_docs_contract.rs` branch-marker assertions so branch counts are informational-only.
5. Run targeted docs-contract lanes and static quality gates.

## Affected Modules
- `docs/review/README.md`
- `docs/review/gaps-and-issues-r50.md`
- `crates/kamn-core/tests/review_r49_docs_contract.rs`
- `crates/kamn-core/tests/review_snapshot_semantics_docs_contract.rs` (new)

## Risks and Mitigations
- Risk: marker parser brittleness across backtick/non-backtick marker formatting.
  - Mitigation: parser trims optional backticks around keys and values.
- Risk: over-constraining historical artifacts.
  - Mitigation: apply snapshot policy guardrails only to R50+ files.
- Risk: accidental drift in existing docs-contract tests.
  - Mitigation: run targeted review docs-contract suites after each phase.

## Interfaces / Contracts
- New policy schema marker:
  - `review_snapshot_semantics_policy_schema_version=kamn.review.snapshot-semantics-policy.v1`
- Required R50+ per-review markers:
  - `r<release>_review_snapshot_as_of_date=<YYYY-MM-DD>`
  - `r<release>_review_branch_remote_head_count_contract_mode=informational_only`
  - `r<release>_review_branch_reconciliation_issue_chain_count=<integer>`
  - `r<release>_review_branch_reconciliation_issue_chain_max=1`
- Optional informational marker:
  - `r<release>_review_branch_remote_head_count_snapshot=<integer>`

## ADR
- Not required.
