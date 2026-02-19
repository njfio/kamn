# Issue #3929 Spec

- Title: Subtask: implement proptest generators and invariants for task and escrow state machines
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Task and escrow lifecycle correctness requires invariant coverage beyond scenario-only fixtures. The task is to ensure generator/invariant surfaces, deterministic replay signals, and bounded-cost guardrails are all enforced and documented.

## Acceptance Criteria
- AC-1: Proptest generators and invariant checks cover task and escrow safety/legal-transition constraints.
- AC-2: Deterministic seed and failure-replay behavior is documented and enforced by tests.
- AC-3: Regression/contract checks fail closed when invariant catalog or split suite wiring drifts.
- AC-4: Bounded-cost property-run envelope remains explicit and validated.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/task_escrow_proptest_invariants/{shared.rs,task_domain.rs,escrow_domain.rs}`
- `crates/kamn-core/tests/task_escrow_suite_modularization_contract.rs`
- `crates/kamn-core/tests/runtime_watchdog_attestation_docs.rs`
- `docs/foundation/runtime-watchdog-attestation.md`
- `specs/3929/{spec.md,plan.md,tasks.md}`

Out of scope:
- Production runtime behavior changes
- Full-scale fuzz campaign expansion
- Shell/workflow/governance script additions

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Task lifecycle transition sequences | Legal transitions accepted; illegal transitions reject without state/history corruption |
| C-02 | AC-1 | Integration | Escrow transition action sequences | Conservation and status-projection invariants hold across bounded runs |
| C-03 | AC-2 | Unit | Deterministic config and seed resolution | Fixed seed/config + persistence settings remain stable |
| C-04 | AC-2 | Regression | Seed corpus replay contract | Tracked proptest regression corpus remains present and loaded |
| C-05 | AC-3 | Conformance | Modularization/doc contract tests | Missing module/docs markers fail closed |
| C-06 | AC-4 | Performance | Property suite budget markers/constants | Case counts and sequence bounds stay inside declared envelope |

## Test Mapping
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo test -p kamn-core --test task_escrow_suite_modularization_contract`
- `cargo test -p kamn-core --test runtime_watchdog_attestation_docs`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Task and escrow proptest suite enforces deterministic and bounded invariants through explicit tests.
- Docs-contract coverage pins required invariant-catalog markers.
- Targeted test and lint gates pass with no shell-surface growth.
