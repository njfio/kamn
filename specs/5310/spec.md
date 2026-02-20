# Spec — Issue #5310

Status: Reviewed (agent-authored; human review requested in PR)
Issue: #5310
Milestone: R27.44 Shell LOC deletion wave and hard ceiling governance

## Problem Statement

The request-response schema compatibility lane introduced in #4042 left a positive shell LOC delta against its merge-base snapshot. We need a concrete mitigation that reduces non-symlink shell LOC while preserving existing CI behavior and shell-surface contract checks.

## Scope

In scope:
- Reduce non-symlink shell LOC by consolidating tiny CI wrapper scripts into the existing exec-dispatch registry/symlink model.
- Preserve wrapper behavior and argument forwarding.
- Add explicit test coverage for dispatch argument-prefix root expansion needed by migrated wrappers.

Out of scope:
- Broad dispatcher redesign.
- Deleting large lane families.
- Any protocol or runtime Rust behavior changes.

## Acceptance Criteria

AC-1: Wrapper consolidation
- Given selected tiny CI wrappers, when mitigation is applied, then they are migrated to `scripts/lib/exec_dispatch.sh` + `scripts/lib/exec_registry.json` entries without behavior regressions.

AC-2: Dispatch root placeholder support
- Given registry `args_prefix` entries that include `${KAMN_ROOT}`, when dispatcher executes, then the placeholder is expanded to absolute repo root before target execution.

AC-3: Guardrail improvement
- Given shell LOC baseline at `#4042` merge-base snapshot, when mitigation is applied, then `shell_loc_delta_actual <= 0` and `shell_to_rust_ratio_delta_actual <= 0.0`.

AC-4: Verification coverage
- Given migrated wrappers and dispatcher changes, when tests run, then unit/functional/integration/regression checks pass for dispatch registry contracts and shell guardrails.

## Conformance Cases

C-01 (AC-1, Unit/Functional):
- Input: migrated wrapper invocation with passthrough args.
- Expected: target Python script receives expected subcommand/prefix args and passthrough args.

C-02 (AC-2, Regression):
- Input: registry entry with `${KAMN_ROOT}` in `args_prefix`.
- Expected: resolved command contains absolute repo-root path for the token.

C-03 (AC-3, Integration):
- Input: shell LOC/rust ratio telemetry before/after mitigation.
- Expected: non-positive deltas vs baseline snapshot used by #4042.

C-04 (AC-4, Regression):
- Input: existing `scripts/lib/test_exec_dispatch_registry.sh` + wrapper-specific checks.
- Expected: all pass with migrated wrappers and registry entries.

## Observable Signals

- `bash scripts/lib/test_exec_dispatch_registry.sh` passes.
- Migrated wrappers execute successfully with existing CLI surfaces.
- Shell LOC check and shell-rust ratio guardrail remain `GO`.

