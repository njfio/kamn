# Spec — Issue #4831

- Title: Subtask: implement shell-rust LOC telemetry collector and fail-closed reason taxonomy outputs
- Parent: Parent task: #4817
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Implemented
- Priority: P1

## Objective

Add a deterministic shell-vs-Rust LOC telemetry collector that emits fail-closed decision markers and a stable reason taxonomy surface.

## Problem Statement

Combined shell-surface trend generation existed, but no single fail-closed telemetry wrapper exposed stable GO/NO-GO markers and taxonomy-coded failure paths for governance consumers.

## Scope

In scope:
- add `collect_shell_rust_loc_telemetry.sh` fail-closed collector wrapper
- emit deterministic reason taxonomy/version and normalized reason markers
- add deterministic pass/fail contract tests for telemetry collection
- wire telemetry collector tests into CI tools regression entrypoint
- update CI strategy docs with collector contract markers

Out of scope:
- CI fast-gate workflow wiring changes (handled in `#4832`)
- threshold policy redesign for combined shell-surface checker

## Acceptance Criteria

- AC-1: Telemetry collector emits deterministic `status`, `final_decision`, taxonomy version, and reason-code markers with shell/Rust metrics.
- AC-2: Collector contract tests cover both passing path and deterministic failing path.
- AC-3: CI regression entrypoint includes collector contract tests so marker drift fails closed.

## Conformance Cases

- C-01 (AC-1, Functional): `bash scripts/ci/collect_shell_rust_loc_telemetry.sh --output-json /tmp/shell-rust-loc-telemetry-report.json` emits `status=ok`, `final_decision=GO`, and `reason_codes=none` for repository baseline.
- C-02 (AC-2, Conformance): `bash scripts/ci/test_collect_shell_rust_loc_telemetry.sh` validates pass markers and deterministic `shell_rust_loc_telemetry_report_missing` NO-GO failure mapping.
- C-03 (AC-3, Integration/Regression): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes with telemetry collector contract test wired in.

## Success Metrics / Signals

- Telemetry collector report schema `kamn.ci.shell-rust-loc-telemetry-report.v1` is generated deterministically.
- Failures emit reason taxonomy `kamn.ci.shell-rust-loc-telemetry-reason-taxonomy.v1` and fail closed.
- CI tools regression includes telemetry collector checks by default.
