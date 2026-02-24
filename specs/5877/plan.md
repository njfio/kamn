# Plan: Issue #5877 - Production Expect Checker Test-Only Path Hardening

- Issue: #5877
- Spec: `specs/5877/spec.md`
- Last Updated: 2026-02-24

## Approach
1. Reproduce the current failure state from `scripts/ci/check_no_production_expect.sh`.
2. Add a focused regression case to `scripts/ci/test_check_no_production_expect.sh` that asserts `src/**/tests.rs` is excluded.
3. Update `scripts/ci/check_no_production_expect.py` path exclusion policy to classify `tests.rs` as test-only artifacts.
4. Re-run checker tests and targeted crate tests as verification evidence.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`
- `specs/5877/*`

## Risks / Mitigations
- Risk: over-broad exclusion could hide real production violations.
- Mitigation: preserve existing fail tests for non-test production files and add explicit regression for `src/**/tests.rs` only.

## Interfaces / Contracts
- Deterministic output fields in checker report remain unchanged:
  - `status`
  - `reason_codes_value`
  - `reason_class`
  - `runtime_panic_replacement_evidence_*`
- Reason taxonomy remains `kamn.ci.production-panic-replacement-reason-taxonomy.v1`.

## ADR
No ADR required (no dependency, protocol, or architecture changes).
