# Issue #4013 Spec

- Title: Task: implement cross-store replay consistency checker with deterministic divergence taxonomy
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

Replay divergence across runtime, channel, message lifecycle, and task operation snapshots can remain silent without an explicit cross-store checker. Promotion and crash-recovery workflows need deterministic divergence reason markers that are stable and policy-verifiable.

## Acceptance Criteria

- AC-1: `kamn-core` exposes a production checker API that evaluates cross-store replay consistency across runtime/channel/message/task snapshot inputs and returns deterministic status and reason markers.
- AC-2: checker divergence classification is deterministic and explicitly grouped into stable classes for:
  - snapshot presence drift,
  - snapshot schema-version drift,
  - runtime cursor/state continuity drift,
  - cross-store cardinality drift.
- AC-3: checker emits a stable reason taxonomy version marker plus deterministic reason-code set suitable for policy/docs parity checks.
- AC-4: unit, functional, integration, and regression tests cover checker behavior, class projection, and taxonomy drift protections.
- AC-5: `docs/foundation/runtime-network.md` documents the checker contract markers and divergence taxonomy; docs-contract tests fail closed on marker drift.
- AC-6: touched suites pass targeted fmt/clippy/tests.

## Scope

In scope:
- New cross-store replay consistency checker production module/API in `kamn-core`.
- Deterministic divergence class/reason projection and taxonomy markers.
- New test coverage (unit/functional/integration/regression) for checker behavior.
- Runtime-network docs marker updates plus docs-contract assertions.

Out of scope:
- Historical replay analytics/search interfaces.
- New external dependencies.
- CI pipeline/wiring changes beyond documenting checker contract markers.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | all four snapshots present and valid | checker status is `Consistent`, reason code is `none` |
| C-02 | AC-2 | Functional | one/more snapshots missing | deterministic presence-drift reason code + class |
| C-03 | AC-2 | Functional | schema-version mismatch in channel/message/task snapshot | deterministic schema-drift reason code + class |
| C-04 | AC-2 | Functional | runtime cursor/state continuity violation | deterministic runtime-continuity reason code + class |
| C-05 | AC-2 | Functional | aggregate domain record count exceeds runtime cursor | deterministic cardinality-drift reason code + class |
| C-06 | AC-3/AC-4 | Unit/Regression | taxonomy version/csv projection | stable taxonomy marker and reason-code list |
| C-07 | AC-4 | Integration | load snapshots from runtime/channel/message/task stores then evaluate checker | composed cross-store checker output remains deterministic |
| C-08 | AC-5 | Conformance | runtime-network docs marker assertions | docs contract fails closed when checker markers drift |
| C-09 | AC-6 | Quality | targeted fmt/clippy/tests | no formatting/lint/regression failures |

## Test Mapping

- `cargo test -p kamn-core --test cross_store_replay_consistency`
- `cargo test -p kamn-core --test runtime_network_docs doc_contains_cross_store_replay_consistency_checker_taxonomy_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals

- Checker API is exported by `kamn-core` and consumable by runtime governance lanes.
- Divergence reports carry deterministic, stable reason code/class/taxonomy markers.
- Docs/test parity fails closed if taxonomy markers drift.
