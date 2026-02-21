# Issue #4078 Plan — Deletion Docs/Runbook Parity Contracts

## Approach
1. Add deletion docs/runbook parity + remediation section in `docs/ci/strategy.md` using the same
   pattern as fairness/overload parity sections.
2. Extend deletion marker content in `docs/ops/configuration.md` with remediation map entries.
3. Add `ci_strategy_docs` tests for:
   - parity marker presence,
   - ops/fixture/strategy taxonomy synchronization,
   - remediation marker coverage for each reason code.
4. Execute RED->GREEN using new docs tests and verify with fmt/clippy/targeted docs suites.

## Affected Modules
- `docs/ci/strategy.md`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4078/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: reason-code list diverges across fixture, ops docs, and strategy docs.
  - Mitigation: direct assertions against shared constant + fixture and ops markers.
- Risk: remediation map becomes partial.
  - Mitigation: loop over reason codes and require remediation entries in both docs.
- Risk: overlap with #4077 docs section.
  - Mitigation: keep `#4078` strictly focused on parity/remediation contract layer.

## Interfaces / Contracts
- Shared reason taxonomy version:
  `kamn.runtime.deletion-proof-checker-reason-taxonomy.v1`.
- Shared reason-code CSV:
  `deletion_proof_subject_missing,deletion_proof_tombstone_missing,deletion_proof_status_invalid,deletion_proof_hash_mismatch`.
- Remediation map key format:
  `deletion_docs_parity_remediation.<reason_code>=...` in strategy docs and ops docs.

## Validation Strategy
- RED: add `ci_strategy_docs` deletion parity test before strategy section exists.
- GREEN: add strategy/ops markers and rerun targeted docs tests.
- VERIFY: `cargo fmt --check`, `cargo clippy -- -D warnings`, targeted docs tests.
