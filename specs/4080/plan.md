# Issue #4080 Plan — Redaction Policy Checker and Drift Controls

## Approach
1. Add checker wrapper `scripts/runtime/check_local_heavy_redaction_validation_policy.sh` and
   map it in `scripts/lib/exec_registry.json`.
2. Implement checker logic in `scripts/runtime/local_heavy_redaction_validation_policy_contract.py`:
   - validate redaction runner schema/reason markers and required fields,
   - enforce expected final decision and ci-fast-gate semantics,
   - verify docs parity markers in `docs/ops/configuration.md` and `docs/ci/strategy.md`,
   - output deterministic policy report schema + reason codes.
3. Add shell contract test
   `scripts/runtime/test_check_local_heavy_redaction_validation_policy.sh` for drift fixtures.
4. Add Rust checker contract tests in
   `crates/kamn-core/tests/local_heavy_redaction_validation_policy_contract.rs`.
5. Add CI strategy policy-checker section and `ci_strategy_docs` marker/parity tests.

## Affected Modules
- `scripts/runtime/check_local_heavy_redaction_validation_policy.sh`
- `scripts/runtime/local_heavy_redaction_validation_policy_contract.py`
- `scripts/runtime/test_check_local_heavy_redaction_validation_policy.sh`
- `scripts/lib/exec_registry.json`
- `crates/kamn-core/tests/local_heavy_redaction_validation_policy_contract.rs`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4080/{spec,plan,tasks}.md`

## Risks / Mitigations
- Risk: checker reason taxonomy drifts from strategy markers.
  - Mitigation: assert strategy markers + checker constants in `ci_strategy_docs`.
- Risk: ops docs markers drift from runner assumptions.
  - Mitigation: checker + integration tests require ops marker parity.
- Risk: policy checker grows expensive.
  - Mitigation: keep checker file-read and JSON-parse only; enforce performance test budget.

## Interfaces / Contracts
- Policy report schema:
  `kamn.runtime.local-heavy-redaction-validation-policy-report.v1`
- Policy reason taxonomy:
  `kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1`
- Deterministic policy reason codes:
  `redaction_policy_required_field_missing,redaction_policy_marker_mismatch,redaction_policy_reason_taxonomy_mismatch,redaction_policy_profile_contract_mismatch,redaction_policy_docs_marker_parity_mismatch,ci_fast_gate_failed,redaction_policy_expected_decision_mismatch,redaction_policy_violation`

## Validation Strategy
- RED: add checker contract/docs-parity tests before checker/strategy section exists.
- GREEN: implement checker + strategy markers + parity assertions and rerun targeted suites.
- VERIFY: targeted tests + `cargo fmt --check` + `cargo clippy -- -D warnings`.
