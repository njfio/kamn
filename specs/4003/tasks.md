# Issue #4003 Tasks

- T1 (Tests first): Add failing docs-parity and remediation-missing cases in:
  - `scripts/ci/test_check_performance_thresholds.sh`
  - `crates/kamn-core/tests/performance_ci_smoke_governance_contract.rs`
  - `crates/kamn-core/tests/ci_strategy_docs.rs`
- T2 (Implementation): Extend `scripts/ci/performance_smoke_contracts.py` with docs marker/remediation checks and deterministic output markers.
- T3 (Docs): Update `docs/ci/strategy.md` performance smoke governance section with remediation map markers for each reason code.
- T4 (Verification): Run targeted shell/Rust tests, then `cargo fmt --check` and `cargo clippy -- -D warnings`.
