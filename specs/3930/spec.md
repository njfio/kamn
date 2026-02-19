# Issue #3930 Spec

- Title: Subtask: implement proptest invariants for peer lifecycle transitions and anti-churn guards
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Peer lifecycle transition correctness and anti-churn rejection behavior must stay deterministic under generated event sequences. The lane needs explicit bounded-budget and docs-contract markers so drift fails closed.

## Acceptance Criteria
- AC-1: Generated peer lifecycle sequences enforce legal transition graph and reject illegal transitions deterministically.
- AC-2: Anti-churn behavior for repeated invalid lifecycle events is covered with deterministic failure reporting.
- AC-3: Deterministic seed/replay behavior is documented and enforced through tests.
- AC-4: Docs-contract checks fail closed on peer lifecycle invariant marker drift.
- AC-5: Bounded property-run envelope is explicit and validated.

## Scope
In scope:
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
- `crates/kamn-core/tests/runtime_network_docs.rs`
- `docs/foundation/runtime-network.md`
- `specs/3930/{spec.md,plan.md,tasks.md}`

Out of scope:
- Production runtime behavior changes
- Network-scale external simulation
- Shell/workflow lane additions

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Generated lifecycle event sequences | Legal transitions apply, illegal transitions reject with stable error semantics |
| C-02 | AC-2 | Integration | Prefix + repeated invalid event | Repeated invalid events stay idempotent and do not mutate state |
| C-03 | AC-3 | Unit | Deterministic config/seed + corpus replay checks | Seed, config, and corpus contracts remain stable |
| C-04 | AC-4 | Regression | Runtime-network docs-contract assertions | Missing peer lifecycle invariant markers fail closed |
| C-05 | AC-5 | Performance | Budget-envelope unit assertion + docs markers | Case and sequence bounds remain inside declared envelope |

## Test Mapping
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`
- `cargo test -p kamn-core --test runtime_network_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Peer lifecycle invariant and anti-churn guarantees are pinned by deterministic tests.
- Runtime-network docs include explicit peer lifecycle property-lane contracts.
- Targeted tests and lint gates pass with no shell-surface growth.
