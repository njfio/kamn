# Move PR Gates To Local Pre-Push

## Objective

Move the heavyweight `ci-fast-gate` PR enforcement out of GitHub Actions while
preserving the same gate intent as an explicit local pre-push command. The
change exists because the GitHub Fast Gate job is cancelled by the hosted
runtime budget even after the local quality gate and longer Workspace
Pre-Merge lane prove the code path.

## Inputs/Outputs

Inputs:

- Issue #7036.
- PR #7022 gate-recovery branch.
- `.github/workflows/ci-fast-gate.yml`, currently defining the Fast Gate,
  CI Tool Regression Gate, and Workspace Pre-Merge Gate PR jobs.
- Existing local verification commands:
  - `cargo fmt --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `make check`
  - `make ci-tools`
  - `cargo test --workspace --locked --all-features --no-fail-fast`
  - critical-path coverage and mutation gate scripts.

Outputs:

- No GitHub Actions workflow file that schedules `ci-fast-gate` PR jobs for
  new PR heads.
- A local make target that runs the pre-push gate sequence explicitly.
- Contract coverage that fails before the workflow removal/local target change
  and passes after it.
- Updated issue/spec evidence for local verification.

## Boundaries/Non-goals

- Do not add MVP demo features in this issue.
- Do not change MVP proof-report schemas or claim taxonomy.
- Do not remove the local gate scripts that the GitHub workflow previously
  called.
- Do not weaken Rust tests, lint levels, clippy strictness, formatting checks,
  critical-path proof scripts, or source-marker assertions.
- Do not fake or simulate settlement, escrow, exchange, or asset movement
  claims.
- Do not push directly to `main`.

## Failure Modes

- The GitHub `ci-fast-gate` workflow remains present and continues scheduling
  PR gate jobs on new heads.
- The local pre-push command omits strict formatting, clippy, CI-tool
  regression, workspace test, touched-size, coverage, or mutation checks.
- The local command returns success after a failed gate.
- Existing proof and policy scripts are deleted or relaxed instead of being
  moved behind the local command.
- Documentation or contract tests still describe the heavyweight gates as
  GitHub-enforced.

## Acceptance Criteria

- [ ] `ci-fast-gate` no longer appears as a GitHub Actions workflow file.
- [ ] A documented local command, `make pre-push`, runs the gate sequence before
      publishing changes.
- [ ] `make pre-push` includes formatting, strict clippy, `make ci-tools`,
      full workspace tests, touched Rust size policy, critical-path coverage,
      and critical-path mutation checks.
- [ ] Tests/contracts fail before the workflow/local target change and pass
      after it.
- [ ] `cargo fmt --check` passes.
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
      passes.
- [ ] `make check` passes.
- [ ] The local pre-push command passes or any failure is separately filed with
      evidence.

## Files To Touch

- `specs/7036-move-pr-gates-to-local-pre-push.md`
- `Makefile`
- `.github/workflows/ci-fast-gate.yml`
- Focused workflow/local-gate contract tests under `scripts/ci/**` or existing
  Rust workflow contract tests.
- Docs or tests that explicitly require `ci-fast-gate` to remain GitHub
  scheduled.

## Error Semantics

- The local pre-push command must fail on the first failed gate command.
- The local pre-push command must not swallow child-process failures.
- Existing shell gate scripts must keep their current fail-closed behavior and
  success markers.
- Removing GitHub scheduling must not turn proof failures into warnings.

## Test Plan

Red:

- Add or update a contract that expects `.github/workflows/ci-fast-gate.yml` to
  be absent or inert and observes the current workflow file as a failure.
- Add or update a contract that expects `make pre-push` to include every local
  gate command and observes the current missing target as a failure.

Green:

- Remove GitHub `ci-fast-gate` workflow scheduling by deleting the workflow
  file or making it inert outside GitHub PR checks.
- Add `make pre-push` with the required local gate sequence.
- Update only tests/docs that encode the old GitHub-enforced policy.

Refactor:

- Keep the local target as a thin orchestration layer over existing scripts and
  make targets.
- Do not duplicate workflow script logic in the Makefile when an existing
  script already owns the check.

Integration/Proof:

- `cargo fmt --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `make check`
- `make ci-tools`
- Targeted contract tests added for this issue.
- `make pre-push`, or documented NO-GO evidence if an existing pre-push check
  is already failing for a separately filed reason.

## Shell-Surface DoD

- `shell_loc_delta_actual: TBD`
- `rust_loc_delta_actual: TBD`
- `shell_to_rust_ratio_delta_actual: TBD`
- `shell_surface_ratio_target_status: TBD`
