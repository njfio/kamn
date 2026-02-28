# Issue 6259 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6256

## Objective
Migrate a bounded `kamn-core` reason-code surface from raw string constants to a
typed enum while preserving string compatibility at wire/telemetry boundaries.

## Inputs/Outputs
Inputs:
- Existing shell-neutral policy reason constants in
  `crates/kamn-core/src/data_layer_shell_neutral_policy.rs`.
- Existing tests in `crates/kamn-core/tests/data_layer_shell_neutral_policy.rs`.
- Root exports in `crates/kamn-core/src/lib.rs`.

Outputs:
- Typed reason enum for shell-neutral policy reason vocabulary.
- Stable conversion helpers for canonical string compatibility.
- Updated tests and callsites to consume typed reason variants.

## Boundaries/Non-goals
In scope:
- Shell-neutral policy reason domain only (`data_layer_shell_neutral_policy`).
- Replace report reason-code vector type with enum variants.
- Provide deterministic enum↔string conversion helpers.

Out of scope:
- Full-repo migration of all `REASON_CODE` constants.
- Runtime behavior changes to policy decision logic.
- Changes to wire string vocabulary.

## Failure modes
- FM-1: String compatibility drift in canonical reason code values.
- FM-2: Consumers fail due missing conversion helpers or export changes.
- FM-3: Invalid reason-code strings map ambiguously instead of failing closed.

## Acceptance criteria (testable booleans)
- AC-1: Shell-neutral policy no longer exposes raw public reason constants as the
  primary reason contract; typed enum is used in policy report surface.
- AC-2: Typed reason enum provides deterministic canonical string conversion
  (`as_str`) and parsing (`FromStr` or equivalent) with fail-closed rejection.
- AC-3: Existing shell-neutral policy tests pass using typed reason assertions.
- AC-4: Canonical reason strings emitted at boundaries remain unchanged.

## Files to touch
- `crates/kamn-core/src/data_layer_shell_neutral_policy.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_shell_neutral_policy.rs`
- `specs/6259/spec.md`
- `specs/6259/plan.md`
- `specs/6259/tasks.md`

## Error semantics
- Parsing unknown reason strings must fail closed with explicit typed error.
- Existing policy evaluation errors remain unchanged.

## Test plan
Conformance:
- C-01: `cargo test -p kamn-core --test data_layer_shell_neutral_policy -- --nocapture` passes.
- C-02: New tests verify enum `as_str` returns canonical legacy reason strings.
- C-03: New tests verify parsing invalid reason string fails deterministically.

Regression:
- `cargo test -p kamn-core --test data_layer_shell_neutral_policy -- --nocapture`
- `cargo test -p kamn-core --lib data_layer_shell_neutral_policy -- --nocapture`
