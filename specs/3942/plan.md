# Issue #3942 Plan

- Issue: #3942
- Status: In Progress
- Spec: `specs/3942/spec.md`

## Implementation Approach
1. Add a checker regression fixture to `scripts/ci/test_check_no_production_expect.sh` that currently slips past scan coverage due top-level `#[cfg(test)]` truncation.
2. Refactor checker source scanning in `check_no_production_expect.py` to skip cfg(test)-guarded items rather than truncating the entire file.
3. Keep deterministic reason-code taxonomy and runtime evidence output contracts unchanged.
4. Re-run checker harness and verify pass/fail fixtures remain deterministic.

## Affected Modules
- `scripts/ci/check_no_production_expect.py`
- `scripts/ci/test_check_no_production_expect.sh`

## Risks and Mitigations
- Risk: simplistic Rust item skipping misclassifies complex cfg(test) blocks.
  - Mitigation: implement brace-aware item skipping and keep focused fixture coverage for top-level imports + test modules.
- Risk: taxonomy output drift.
  - Mitigation: keep reason-code constants untouched and run existing fixture assertions.

## Contracts and Interfaces
- Reason taxonomy constants remain:
  - `kamn.ci.production-panic-replacement-reason-taxonomy.v1`
  - `scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default`
- Runtime evidence markers remain:
  - `runtime_panic_replacement_evidence_status`
  - `runtime_panic_replacement_evidence_violation_count`
  - `runtime_panic_replacement_evidence_files_csv`

## Verification Strategy
- RED: add top-level cfg(test) regression fixture and observe failure before checker fix.
- GREEN: update checker scanning logic to detect production violation after cfg(test) items.
- REGRESSION: run full checker harness script and verify deterministic outputs.
