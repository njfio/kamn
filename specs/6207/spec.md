# Spec: Issue #6207 - AgentIdentity Secret Material Zeroization + Clone Removal

- Status: Implemented
- Priority: P2
- Parent: #6183
- Milestone: R59 Swarm Gap Closure

## Problem Statement

`AgentIdentity` currently derives `Clone` and stores signing-key material in plain `String` fields
without deterministic zeroization on drop. This allows uncontrolled key copies and increases the
residual lifetime of sensitive key bytes in memory.

## Scope

In scope:
- Remove `Clone` derivation from `AgentIdentity`.
- Add deterministic zeroization for signing/encryption key buffers on drop.
- Add regression tests to fail closed if clone reintroduction or zeroization marker drift occurs.

Out of scope:
- Full key custody redesign across crates.
- Transport/auth protocol changes.

## Acceptance Criteria

### AC-1 Clone Risk Removal
Given `AgentIdentity`,
When type derives are inspected,
Then `Clone` is not derived.

### AC-2 Deterministic Zeroization
Given `AgentIdentity` instances are dropped,
When drop path executes,
Then key material buffers are zeroized.

### AC-3 Regression Lock
Given future refactors,
When regression tests run,
Then clone/zeroization markers remain fail-closed.

## Conformance Cases

- C-01 (AC-1, Unit): source contract test verifies `AgentIdentity` does not derive `Clone`.
- C-02 (AC-2, Unit): source contract test verifies `Drop` implementation invokes key zeroization.
- C-03 (AC-3, Regression): targeted `kamn-agent-lib` tests pass with new invariants.

## Success Metrics

- `cargo test -p kamn-agent-lib` remains green.
- No `AgentIdentity` clone usage remains in crate tests/source.
