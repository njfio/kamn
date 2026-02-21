# Issue #4077 Plan — Deletion-Proof Fixture and Checker Contracts

## Approach
1. Add deletion-proof fixture matrix under `fixtures/runtime/` with schema/taxonomy metadata and
   pass/fail proof-class rows.
2. Add checker contract test file in `kamn-core` to parse fixture rows and evaluate deterministic
   fail-closed behavior.
3. Add ops docs marker section and docs-parity assertions in
   `service_api_ops_configuration_docs.rs`.
4. Execute RED->GREEN and verify with fmt/clippy/targeted tests.

## Affected Modules
- `fixtures/runtime/deletion_proof_artifact_fixture_matrix.txt`
- `crates/kamn-core/tests/deletion_proof_artifact_checker_contract.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4077/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: checker behavior drifts from fixture expectations.
  - Mitigation: integration contract checks every row's expected status/reason.
- Risk: docs marker drift from fixture taxonomy.
  - Mitigation: dedicated docs parity test asserting all deletion-proof markers and command.
- Risk: overlap with follow-up docs-runbook issue `#4078`.
  - Mitigation: keep this issue scoped to fixture+checker behavior + ops marker map only.

## Interfaces / Contracts
- Fixture row columns:
  `case_id|subject_id|tombstone_hash|expected_hash|proof_status|expected_status|expected_reason_code`.
- Deterministic fail-closed reasons:
  `deletion_proof_subject_missing`, `deletion_proof_tombstone_missing`,
  `deletion_proof_status_invalid`, `deletion_proof_hash_mismatch`.
- Pass condition:
  non-empty subject/tombstone hash, `proof_status=deleted`, and
  `tombstone_hash == expected_hash`.

## Validation Strategy
- RED: add docs-parity assertion for deletion-proof section before docs update.
- GREEN: add fixture/checker/docs markers and rerun targeted tests.
- VERIFY: `cargo fmt --check`, `cargo clippy -- -D warnings`, and targeted contract suites.
