# Spec: Issue #6132 - Integer-only watchdog delivery ratio

Status: Accepted
Issue: #6132
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`WatchdogNode::observe_gossip` computes delivery ratio percent using `f64` and floor conversion. The R59 swarm review flagged this (`S-09`) as a determinism risk for blockchain/runtime guard contexts. We need integer-only ratio math with equivalent floor semantics.

## Scope
In scope:
- Replace the `f64`-based ratio calculation in `crates/kamn-runtime-guards/src/watchdog.rs` with integer math.
- Add conformance tests covering deterministic floor behavior and threshold boundary behavior.
- Preserve existing watchdog API and alert taxonomy.

Out of scope:
- Changes to watchdog config shape.
- Changes to alert payload schema.
- Broader runtime-guards refactors.

## Acceptance Criteria
- AC-1: Delivery ratio percentage in watchdog censorship checks is computed with integer arithmetic only (no floating-point operations).
- AC-2: Ratio threshold behavior is deterministic and floor-based for non-divisible ratios.
- AC-3: Existing watchdog validation and classification behavior remains unchanged outside the ratio implementation detail.

## Conformance Cases
- C-01 (AC-1): Source contains no `f64`-based ratio conversion in `observe_gossip`; ratio is derived from integer arithmetic.
- C-02 (AC-2): For delivered/expected of `2/3`, observed ratio is `66`, and threshold `67` yields a censorship warning.
- C-03 (AC-2): For delivered/expected of `7/10`, observed ratio is `70`, and threshold `70` yields no warning.
- C-04 (AC-3): Existing invalid-observation guards (e.g., delivered > expected) continue to return the same error.

## Success Metrics
- `cargo test -p kamn-runtime-guards watchdog::tests::`
- `cargo test -p kamn-runtime-guards`
- `cargo clippy -p kamn-runtime-guards --tests -- -D warnings`
