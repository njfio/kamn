# Issue #4136 Spec

- Title: Subtask: implement invariant helper library and deterministic seed configuration for property runners
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Property runners currently duplicate deterministic configuration and transition-invariant helper logic. The duplication increases drift risk and makes seed policy changes harder to apply consistently.

## Acceptance Criteria
- AC-1: Property runner seed configuration is deterministic and configurable through standardized helper APIs.
- AC-2: Shared helper APIs encode common lifecycle transition invariants used by property suites.
- AC-3: Integration/property suites use the helper library and remain reproducible.
- AC-4: Runtime state-model documentation references the invariant helper layer.
- AC-5: Existing and new deterministic property tests pass.

## Scope
In scope:
- `crates/kamn-core/tests/task_escrow_proptest_invariants.rs`
- `crates/kamn-core/tests/peer_lifecycle_proptest_invariants.rs`
- `crates/kamn-core/tests/property_invariant_helpers.rs` (new)
- `crates/kamn-core/tests/property_invariant_helpers_contracts.rs` (new)
- `docs/architecture/runtime-state-model.md`
- `specs/4136/{spec.md,plan.md,tasks.md}`

Out of scope:
- New external dependencies
- Runtime production behavior changes
- Fuzz or concurrency lane redesign

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | Seed override parser inputs (`None`, valid numeric, invalid) | Deterministic default/override behavior with invalid fallback |
| C-02 | AC-2 | Unit | Task/peer transition pairs | Helper APIs classify legal/illegal transitions deterministically |
| C-03 | AC-3 | Integration | Existing property suites wired to shared helper module | Suites remain deterministic and pass |
| C-04 | AC-4 | Regression | Runtime state-model docs assertions | Docs contain invariant helper references |
| C-05 | AC-5 | Regression | Full targeted property helper + suite runs | All pass with stable seeds |

## Test Mapping
- `cargo test -p kamn-core --test property_invariant_helpers_contracts`
- `cargo test -p kamn-core --test task_escrow_proptest_invariants`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Seed policy updates can be applied in one shared helper location.
- Property runners consume shared invariant helper APIs instead of local duplicated logic.
- Deterministic behavior is preserved and verified.
