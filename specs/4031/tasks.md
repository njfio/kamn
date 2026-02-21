# Issue #4031 Tasks

- T1 (Tests first): add docs-contract assertions for dependency checker threshold/remediation
  markers and capture RED failure.
- T2 (Implementation): add dependency CI smoke checker module + public exports and checker contract
  tests.
- T3 (Docs/CI wiring): update `docs/ci/strategy.md` markers and wire checker contract test command
  in `scripts/ci/test_ci_tools.sh` fast/full paths.
- T4 (Verification): run targeted checker/docs/script tests plus `cargo fmt --check` and
  `cargo clippy -p kamn-core --tests -- -D warnings`.
- T5 (Process): open PR with AC mapping/TDD evidence and close issue with status+outcome markers.
