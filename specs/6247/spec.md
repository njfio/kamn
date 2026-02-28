# Issue 6247 Spec

Status: Implemented
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
Critical-path coverage thresholds remain too weak for several high-risk files (for example, `signer.rs` line 4.5% and function 8.5%), reducing CI assurance value and allowing major behavior gaps to pass merge gates.

## Scope
In scope:
- Raise critical-path thresholds in `.ci/critical-path-coverage-thresholds.json` with explicit rationale.
- Add/expand tests needed to satisfy stricter minima.
- Keep coverage gate deterministic and fail-closed.

Out of scope:
- Replacing coverage tooling.
- Expanding target-file inventory beyond currently gated paths.

## Acceptance Criteria
- AC-1: Every target in `.ci/critical-path-coverage-thresholds.json` has strictly higher line and function minima than the current baseline.
- AC-2: The three weakest baseline targets (`signer.rs`, `runtime_orchestration.rs`, `kolme_runtime_commit/http_transport.rs`) each gain at least +10 absolute points in either line or function minimum.
- AC-3: Added/updated tests make the stricter thresholds pass without lowering new minima.
- AC-4: Coverage gate remains fail-closed with deterministic reason taxonomy when thresholds are violated.

## Conformance Cases
- C-01 (AC-1, Conformance): Threshold JSON diff shows increased minima for all listed targets.
- C-02 (AC-2, Conformance): Threshold values for the three weakest targets meet the +10 absolute-point requirement on at least one dimension.
- C-03 (AC-3, Functional): Targeted `cargo test` and coverage gate execution pass with updated minima.
- C-04 (AC-4, Regression): Policy script tests/assertions verify deterministic fail-closed behavior is unchanged.

## Success Metrics
- Critical-path threshold floor is materially higher across all gated files.
- CI coverage failures become actionable for risky modules instead of nominal.
