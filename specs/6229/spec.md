# Issue 6229 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
Current critical-path coverage thresholds are too low to be materially useful (for example, 2-2.5% on signer path), which weakens CI assurance signal quality. Thresholds must be ratcheted upward using measured baseline coverage while keeping the pre-merge gate deterministic.

## Scope
In scope:
- Re-baseline `.ci/critical-path-coverage-thresholds.json` with staged, meaningful increases per target.
- Document threshold rationale and ratchet policy for each target update.
- Verify critical-path coverage gate remains deterministic and fail-closed.

Out of scope:
- Expanding critical-path target list beyond the currently gated files.
- Replacing coverage tooling (`cargo llvm-cov`) or gate architecture.

## Acceptance Criteria
- AC-1: Threshold file is updated with higher minimums for at least one target in both `kamn-core` and `kamn-node` sets.
- AC-2: Threshold changes include explicit rationale in repository docs/spec artifacts.
- AC-3: Coverage gate policy script behavior remains deterministic and fail-closed.
- AC-4: Targeted gate verification passes with the new threshold baseline.

## Conformance Cases
- C-01 (AC-1, Conformance): `.ci/critical-path-coverage-thresholds.json` values increase from previous baseline.
- C-02 (AC-1, Conformance): `kamn-core` threshold entries are ratcheted upward.
- C-03 (AC-1, Conformance): `kamn-node` threshold entries are ratcheted upward.
- C-04 (AC-2, Functional): Documentation/spec includes per-target rationale for new minima.
- C-05 (AC-3, Regression): `scripts/ci/check_critical_path_coverage.py` contract behavior remains unchanged for schema/order/fail-closed semantics.
- C-06 (AC-4, Functional): `bash scripts/ci/run_critical_path_coverage_gate.sh ...` passes using updated thresholds.

## Success Metrics
- Critical-path coverage minima move from placeholder floors to enforcement values that can catch regressions.
- CI output remains actionable (`reason_codes_csv`, failed target counts) under the new baseline.
