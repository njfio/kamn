# Issue 6247 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
Critical-path coverage thresholds in `.ci/critical-path-coverage-thresholds.json` still permit weak minima for security- and runtime-sensitive paths (notably `kamn-node` signer and orchestration paths). This reduces the practical assurance value of the pre-merge coverage gate.

## Scope
In scope:
- Raise threshold minima for all currently-gated critical-path targets to defensible values based on measured current coverage.
- Keep the gate deterministic and fail-closed through existing policy checker wiring.
- Add/update planning documentation with explicit old/new threshold rationale for R59 follow-up tracking.

Out of scope:
- Adding new critical-path target files to the gate.
- Replacing coverage tooling architecture.
- Repository-wide global coverage policy redesign.

## Acceptance Criteria
- AC-1: `.ci/critical-path-coverage-thresholds.json` minimums are increased for each existing target and remain below measured current coverage headroom.
- AC-2: Critical-path coverage gate passes with raised thresholds using current targeted coverage probes.
- AC-3: Critical-path coverage policy remains fail-closed when thresholds are violated.
- AC-4: R59 planning docs include explicit old/new threshold values and rationale.

## Conformance Cases
- C-01 (AC-1, Conformance): Threshold entries for all six current targets are strictly higher than prior baseline values.
- C-02 (AC-1, Functional): Updated minima remain <= currently measured coverage for each target.
- C-03 (AC-2, Integration): `scripts/ci/run_critical_path_coverage_gate.sh` succeeds with updated threshold file.
- C-04 (AC-3, Regression): `scripts/ci/check_critical_path_coverage.py` still emits deterministic fail-closed reason codes for threshold breaches.
- C-05 (AC-4, Functional): `docs/planning/r59-followup.md` records threshold before/after and policy rationale.

## Test Mapping
- Unit: existing checker unit/fixture tests in `scripts/ci/test_check_critical_path_coverage.sh`.
- Functional: threshold file + docs parity checks against generated policy report values.
- Integration: `scripts/ci/run_critical_path_coverage_gate.sh` execution.
- Regression: `scripts/ci/test_check_critical_path_coverage.sh` fail-closed path assertions.
- Performance: N/A (configuration/policy tuning only).
