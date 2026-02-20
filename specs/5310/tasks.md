# Tasks — Issue #5310

Issue: #5310
Spec: `specs/5310/spec.md`
Plan: `specs/5310/plan.md`

- [x] T1 (Red): add/extend dispatch registry tests for `${KAMN_ROOT}` args-prefix expansion and wrapper parity.
- [x] T2 (Green): implement `${KAMN_ROOT}` expansion in `scripts/lib/exec_dispatch.py`.
- [x] T3 (Green): add registry entries and migrate selected tiny CI wrappers to dispatcher symlinks.
- [x] T4 (Refactor): keep registry ordering and wrapper set deterministic.
- [x] T5 (Regression): run `scripts/lib/test_exec_dispatch_registry.sh` and relevant CI wrapper tests.
- [x] T6 (Verify): run shell LOC + shell-rust ratio checks and compute deltas vs #4042 baseline snapshot.
