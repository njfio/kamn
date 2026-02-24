# Tasks: Issue #5889 - sdk_direct Unsafe Env Fallback Regression Remediation

1. T1 (Conformance/RED): run `scripts/ci/check_no_production_expect.sh` and capture current failing violation set.
2. T2 (Implementation): replace remaining direct fallback-default callsites in `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs` using helper-based resolution with identical defaults.
3. T3 (Conformance/GREEN): run `scripts/ci/check_no_production_expect.sh` and `python3 scripts/ci/check_no_production_expect.py --root crates/kamn-e2e-harness/src`.
4. T4 (Regression/Integration): run `cargo test -p kamn-e2e-harness`.
5. T5 (Conformance Evidence): run targeted regex checks proving removal of the prior unsafe fallback-default paths.
