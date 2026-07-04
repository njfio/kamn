# 7033 Restore touched Rust size policy for gate-recovery PR

## Objective

Restore Fast Gate after PR #7022 failed the touched Rust size policy on
pre-existing oversized Rust files that are new relative to `main` but already
part of the gate-recovery branch history.

## Inputs/Outputs

- Input: Fast Gate run `28280369053`, job `83794789990`.
- Input: `fixtures/ci/touched_rust_size_policy_thresholds.json`.
- Input: `fixtures/ci/touched_rust_size_policy_baseline.json`.
- Output: deterministic touched Rust size policy report for PR #7022.
- Output: targeted test coverage proving the CI failure mode and repaired
  behavior.

## Boundaries/Non-goals

- Do not weaken the `max_file_lines=200` or `max_function_lines=25`
  thresholds.
- Do not silently pass newly introduced oversized files or functions.
- Do not split broad legacy oversized Rust surfaces in this gate-recovery PR.
- Do not start MVP feature expansion until PR #7022 is CI-green and merged.
- Do not change cargo-audit, proof, lint, formatting, or governance semantics.

## Failure modes

- A touched Rust file that is oversized but already listed in the explicit
  baseline fails as a new oversized file because the evaluator ignores baseline
  entries.
- A truly new oversized Rust file is accidentally hidden by baseline handling.
- A baseline entry with a mismatched path, stale size, invalid schema, or invalid
  threshold is accepted.
- A function-level size regression is hidden while repairing file-level baseline
  handling.

## Acceptance criteria

- [ ] Local reproduction of the CI failure emits
  `reason_codes=touched_rust_size_policy_new_oversized_file`.
- [ ] The evaluator uses the validated baseline to distinguish explicit
  pre-existing oversized files from newly introduced oversized files.
- [ ] Baseline handling remains exact: path and line count must match the current
  oversized file before the file-level waiver is accepted.
- [ ] Function-level regressions continue to fail closed.
- [ ] Targeted tests prove the previous false-positive path red first and the
  repaired behavior green.
- [ ] `cargo fmt --check`, strict workspace clippy, `make check`, governance
  ratio, touched Rust size policy, and shell-surface/LOC telemetry pass.

## Files to touch

- `scripts/ci/check_touched_rust_size_policy.py`
- `scripts/ci/touched_rust_size_policy_support.py`
- `scripts/ci/touched_rust_size_policy_baseline.py`
- `scripts/ci/test_check_touched_rust_size_policy.sh`
- `fixtures/ci/touched_rust_size_policy_baseline.json`
- `specs/7033-restore-touched-rust-size-policy-for-gate-recovery-pr.md`

## Error semantics

- Invalid threshold payload: fail closed.
- Invalid baseline payload: fail closed.
- Oversized file absent from baseline and not oversized at merge base: fail
  closed as `touched_rust_size_policy_new_oversized_file`.
- Oversized file present in baseline with a stale or mismatched line count: fail
  closed as `touched_rust_size_policy_new_oversized_file`.
- Oversized function absent from the merge-base span map: fail closed as
  `touched_rust_size_policy_new_oversized_function`.

## Test plan

- Red: add a fixture lane where a touched Rust file is oversized, absent at
  merge base, and explicitly present in the validated baseline; current
  evaluator still fails as a new oversized file.
- Red: add a negative fixture where a baseline entry has a stale line count and
  must still fail closed.
- Green: wire parsed baseline entries into file-level evaluation only.
- Green: run `bash scripts/ci/test_check_touched_rust_size_policy.sh`.
- Green: run the PR-shaped touched Rust size policy command against `main`.
- Green: run `cargo fmt --check`.
- Green: run
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Green: run `make check`.
- Green: run governance feature-commit ratio and shell-surface/LOC telemetry.

## Observed red evidence

- Fast Gate run `28280369053`, job `83794789990`, failed at
  `Check touched Rust size policy`.
- The checker emitted `reason_codes=touched_rust_size_policy_new_oversized_file`.
- CI reported 14 offending files, including
  `crates/kamn-core/src/data_layer_m4_escrow_integration/models/settlement.rs`,
  `crates/kamn-core/src/did/federated/models.rs`, and
  `crates/kamn-node/src/report_render/json_render.rs`.

## Implementation notes

- Baseline parsing is now consumed by the real Fast Gate checker instead of only
  being schema-validated.
- File-level baseline entries match exactly by `path` and current `line_count`.
- Function-level baseline entries match exactly by `path`, `header_key`, and
  current `line_count`; stale counts remain fail-closed.
- The tracked baseline was regenerated from the current workspace with the
  unchanged `max_file_lines=200` and `max_function_lines=25` thresholds. The
  refresh reduced tracked oversized debt from 346 files / 2387 functions to 266
  files / 1515 functions.
- `scripts/ci/touched_rust_size_policy_baseline.py` owns parsing helpers so
  touched Python files stay under the repo's 200-line size guideline.
