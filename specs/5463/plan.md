# Issue #5463 Plan - Spec-Volume Guardrail Contracts

## Approach
1. TDD RED: add a new docs-contract test targeting R48 guardrail/coherence marker set before doc changes.
2. Add deterministic marker section to `docs/review/gaps-and-issues-r48.md` with parseable numeric/string values and evidence command markers.
3. GREEN: run focused docs-contract test and supporting review contract suite.
4. Publish process/closure markers with shell-surface DoD as neutral (no shell/workflow/template changes).

## Affected Modules
- `docs/review/gaps-and-issues-r48.md`
- `crates/kamn-core/tests/review_r48_spec_volume_guardrail_docs_contract.rs` (new)
- `specs/milestones/r48-1-spec-volume-coherence-batching/index.md`
- `specs/5463/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: marker duplication or drift between R45 and R48 docs.
  - Mitigation: include explicit schema-version anchors and numeric-consistency assertions in tests.
- Risk: over-constraining future review edits.
  - Mitigation: constrain only deterministic policy markers, not narrative text.

## Interfaces / Contracts
- Spec-volume guardrail marker schema:
  - `spec_volume_guardrail_policy_schema_version=kamn.review.spec-volume-guardrail-policy.v1`
- Coherence carry-forward marker schema:
  - `coherence_contract_batching_policy_schema_version=kamn.review.coherence-contract-batching-policy.v1`

## Validation Strategy
- RED:
  - `cargo test -p kamn-core --test review_r48_spec_volume_guardrail_docs_contract -- --nocapture` (expected fail before marker docs land)
- GREEN/REGRESSION:
  - `cargo test -p kamn-core --test review_r48_spec_volume_guardrail_docs_contract -- --nocapture`
  - `cargo test -p kamn-core --test review_coherence_batching_policy_docs_contract -- --nocapture`
  - `cargo fmt --check`
