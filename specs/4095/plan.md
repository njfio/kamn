# Issue #4095 Plan

## Summary
Close overload governance taxonomy drift by projecting taxonomy markers directly from the stress matrix runner and enforcing them in the existing CI dry-run checker.

## Affected Areas
- `scripts/ci/run_daemon_os_signal_stress_matrix.sh`
- `scripts/ci/check_daemon_os_signal_stress_policy.py`
- `scripts/ci/test_check_daemon_os_signal_stress_policy.py`
- `scripts/ci/test_run_daemon_os_signal_stress_matrix.sh`
- `fixtures/ci/daemon_os_signal_stress_policy_thresholds.env`
- `docs/ci/strategy.md`
- `specs/4095/{spec.md,plan.md,tasks.md}`

## Approach
1. Add deterministic overload taxonomy constants to stress matrix runner and include them in emitted JSON/stdout markers.
2. Extend threshold fixture with expected report taxonomy fields.
3. Extend the existing policy checker to validate:
   - report `reason_taxonomy_version` matches fixture expectation,
   - report `reason_codes_csv` matches fixture expectation,
   - runtime reason_code remains a member of expected reason csv.
4. Expand checker test coverage with explicit taxonomy-version and reason-csv mismatch cases.
5. Update CI strategy documentation to include new threshold keys and failure markers.
6. Run targeted shell and docs-contract tests to verify conformance.

## Risks and Mitigations
- Risk: marker churn breaks downstream tooling.
  - Mitigation: keep marker names/version deterministic and test both pass/fail paths.
- Risk: shell LOC growth.
  - Mitigation: reuse the existing checker/test instead of introducing a new checker family.
- Risk: false positives from strict csv comparison.
  - Mitigation: enforce exact stable csv ordering and document it as contract.

## Interfaces / Contracts
- Stress matrix report JSON additions:
  - `reason_taxonomy_version`
  - `reason_codes_csv`
- Threshold fixture additions:
  - `REPORT_REASON_TAXONOMY_VERSION`
  - `REPORT_REASON_CODES_CSV`
- Checker fail-closed reason codes additions:
  - `overload_policy_report_reason_taxonomy_mismatch`
  - `overload_policy_report_reason_codes_csv_mismatch`

## ADR
Not required (contract hardening within existing checker/report boundary).
