# Issue #3942 Tasks

- Issue: #3942
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add cfg(test)-prefix regression fixture proving current checker false-negative behavior.
- [x] T2 (Green): implement cfg(test)-item skipping scanner in checker Python module.
- [x] T3 (Regression): run `scripts/ci/test_check_no_production_expect.sh` and confirm deterministic reason taxonomy outputs.
- [x] T4 (Verify): ensure no workflow/shell-surface expansion beyond scoped checker/test updates.

## Tier Mapping
- Unit: scanner behavior against top-level cfg(test) + production violation fixture.
- Functional: reason-code and reason-class outputs for panic primitive/unsafe fallback fixtures.
- Integration: full checker harness script pass with baseline + failure fixtures.
- Regression: preserved pass on test-only panic usage fixture and new cfg(test)-prefix violation fixture.
- Performance: N/A (low-cost checker logic update only).
