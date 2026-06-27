# 7026-repair-critical-path-mutation-gate-group-channel-selector

## Objective
Restore the Fast Gate CI-tool regression lane by updating the critical-path
mutation gate's group-channel selector to the current extracted nonce-guard
owner without weakening mutation expectations or escape detection.

## Inputs/Outputs
- Inputs:
  - `scripts/ci/run_critical_path_mutation_gate.sh`
  - `scripts/ci/test_run_critical_path_mutation_gate.sh`
  - `crates/kamn-core/src/group_channel_crypto/engine/sealing/encrypt.rs`
- Outputs:
  - `bash scripts/ci/test_run_critical_path_mutation_gate.sh` passes.
  - The group-channel mutation slice still reports one expected mutant in
    stub mode.
  - The critical-path mutation report contract remains at six slices and ten
    expected mutants.

## Boundaries/Non-goals
- Do not remove, skip, or mark the group-channel mutation slice optional.
- Do not lower expected mutant counts.
- Do not weaken mutation escape detection or final `NO-GO` behavior.
- Do not broaden this issue into MVP demo feature work.
- Do not run full live mutation testing unless the existing script contract
  requires it.

## Failure Modes
- Selector drift causes the script to search a stale parent source file and
  exit before producing a report.
- A broad regex could select the wrong group-channel mutation and make the
  critical-path slice less meaningful.
- A stub-only test could pass while the real file path handed to
  `cargo mutants` remains stale.

## Acceptance Criteria
- [ ] Red evidence captures `bash scripts/ci/test_run_critical_path_mutation_gate.sh`
      failing with `unable to resolve group channel mutation selector line`.
- [ ] A committed contract fails before the selector repair and requires the
      group-channel selector to reference
      `crates/kamn-core/src/group_channel_crypto/engine/sealing/encrypt.rs`.
- [ ] The group-channel mutation selector resolves the current production
      nonce guard in the extracted encrypt module.
- [ ] The bounded mutation gate still expects the `core-group-channel-crypto`
      slice to discover exactly one mutant.
- [ ] The script regression test passes in stub mode and continues to require
      six mutation slices and ten expected mutants.
- [ ] `cargo fmt --check`, strict workspace clippy, and `make check` remain
      green.

## Files To Touch
- `scripts/ci/run_critical_path_mutation_gate.sh`
- `scripts/ci/test_run_critical_path_mutation_gate.sh`
- A Rust contract test under `crates/kamn-core/tests/`

## Error Semantics
- Missing selector resolution remains a hard failure.
- Mutant discovery count mismatch remains a hard failure.
- Mutation slice nonzero exits, missed mutants, unviable mutants, and timeouts
  remain hard failures.

## Test Plan
- Red: run `bash scripts/ci/test_run_critical_path_mutation_gate.sh` and
  capture the stale selector failure.
- Red: add a Rust contract that requires the extracted group-channel selector
  path and observe it fail before the script repair.
- Green: update the selector path and regex to the extracted nonce guard.
- Integration: rerun `bash scripts/ci/test_run_critical_path_mutation_gate.sh`
  and the new Rust contract.

## Completion Evidence
- Pending.

## Shell-Surface Metrics
- `shell_loc_delta_estimate: +20`
- `rust_loc_delta_estimate: +120`
- `shell_to_rust_ratio_delta_estimate: -0.0001`
- `shell_surface_mitigation_issue: #7026`
