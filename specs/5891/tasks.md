# Tasks: Issue #5891 - Expand Default Panic-Path Audit Coverage to kamn-agent-lib

1. T1 (Conformance/RED-or-baseline): run `python3 scripts/ci/check_no_production_expect.py --root crates/kamn-agent-lib/src`.
2. T2 (Implementation): add `crates/kamn-agent-lib/src` to `DEFAULT_RUNTIME_ROOTS`.
3. T3 (Conformance/GREEN): run `scripts/ci/check_no_production_expect.sh`.
4. T4 (Regression): run `scripts/ci/test_check_no_production_expect.sh`.
5. T5 (Conformance): verify root inclusion via `rg` marker scan.
