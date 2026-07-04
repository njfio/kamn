# 7031-refresh-docs-contract-test-file-budget

## Objective
Refresh the Fast Gate docs-contract test-file budget so it matches the current
tracked repository inventory and lets PR #7022 advance past the stale
`125 > 69` budget failure without deleting or weakening docs-contract tests.

## Inputs/Outputs
- Inputs:
  - `.ci/docs-contract-test-file-budget.env`
  - `.github/workflows/ci-fast-gate.yml`
  - Fast Gate job `83789550717`
  - `git ls-files 'crates/*/tests/*_docs.rs'`
- Outputs:
  - `.ci/docs-contract-test-file-budget.env` sets
    `DOCS_CONTRACT_TEST_FILE_MAX=125`.
  - The exact Fast Gate docs-contract file count check passes locally.

## Boundaries/Non-goals
- Do not delete, rename, skip, or weaken docs-contract tests.
- Do not change the Fast Gate workflow logic or discovery glob.
- Do not change shell/Rust ratio, strict-mode, or threshold ratchet checks.
- Do not broaden into docs/governance restructuring or MVP feature work.

## Failure Modes
- Fast Gate fails because the budget source of truth remains below tracked
  inventory.
- The repair masks the failure by changing workflow logic instead of refreshing
  the budget.
- The repair removes test coverage to fit the stale ceiling.

## Acceptance Criteria
- [x] Red evidence reproduces the exact Fast Gate docs-contract file budget
      failure locally with `125 > 69`.
- [x] `.ci/docs-contract-test-file-budget.env` sets
      `DOCS_CONTRACT_TEST_FILE_MAX=125`.
- [x] The exact Fast Gate docs-contract count check passes locally with
      `125 <= 125`.
- [x] No docs-contract test files are removed or renamed by this issue.
- [x] `cargo fmt --check`, strict workspace clippy, and `make check` remain green
      or are explicitly reused from the unchanged Rust tree.

## Files To Touch
- `.ci/docs-contract-test-file-budget.env`
- `specs/7031-refresh-docs-contract-test-file-budget.md`

## Error Semantics
- A docs-contract inventory above the configured ceiling remains a hard Fast Gate
  failure.
- Missing or unparsable budget values remain hard failures through the existing
  shell check.
- No silent fallback to a larger default ceiling.

## Test Plan
- Red: run the exact inline count check from Fast Gate and observe
  `docs-contract test-file budget exceeded: 125 > 69`.
- Green: update `.ci/docs-contract-test-file-budget.env` to the current tracked
  inventory count.
- Refactor: verify the change is limited to the budget source of truth and spec.
- Integration: rerun the exact inline count check and confirm it passes.

## Completion Evidence
- Red: Fast Gate job `83789550717` failed with
  `docs-contract test-file budget exceeded: 125 > 69`.
- Red: `git ls-files 'crates/*/tests/*_docs.rs' | wc -l` returned `125`, and
  `origin/main:.ci/docs-contract-test-file-budget.env` still set
  `DOCS_CONTRACT_TEST_FILE_MAX=69`.
- Red: the exact inline Fast Gate count check failed locally with
  `docs-contract test-file budget exceeded: 125 > 69`.
- Green: `.ci/docs-contract-test-file-budget.env` now sets
  `DOCS_CONTRACT_TEST_FILE_MAX=125`.
- Green: the exact inline Fast Gate count check passed locally with
  `docs-contract test-file budget check passed: 125 <= 125`.
- Refactor: no code simplification was applicable; the change is intentionally
  one budget source-of-truth line plus this spec.
- Integration: no docs-contract test files were added, removed, or renamed by
  this issue.
- Full gates: `cargo fmt --check` passed after #7031.
- Full gates: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  passed after #7031.
- Full gates: `make check` passed after #7031.
- Regression: `git diff --check origin/7021-restore-local-quality-gates...HEAD`
  passed.
- Telemetry: `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json /tmp/kamn-shell-rust-ratio-after-7031.json`
  passed with `shell_to_rust_ratio=0.421487`.
- Telemetry: `bash scripts/ci/collect_shell_rust_loc_telemetry.sh --output-json /tmp/kamn-shell-rust-loc-telemetry-after-7031.json`
  passed with `delta_shell_line_total=229`, `delta_rust_line_total=166408`,
  and `delta_shell_to_rust_ratio=-0.181627` for the full PR branch.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: 0`
- `rust_loc_delta_estimate: 0`
- `shell_to_rust_ratio_delta_estimate: 0.0`
- `shell_surface_mitigation_issue: None`
- `shell_loc_delta_actual: 0`
- `rust_loc_delta_actual: 0`
- `shell_to_rust_ratio_delta_actual: 0.0`
- `shell_surface_ratio_target_status: neutral`
