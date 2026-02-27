# Issue 6219 Spec

Status: Implemented
Priority: P2
Milestone: R59 Swarm Gap Closure
Parent: #6183

## Problem Statement
`kamn-cli` no longer exports `is_help_request` and `render_help_text` from `crates/kamn-cli/src/lib.rs`, while `crates/kamn-cli/src/main.rs` still calls both symbols. This breaks `cargo build --workspace --release` in CI.

## Scope
In scope:
- Restore missing exported helper functions in `kamn-cli` crate root.
- Ensure helper behavior matches current parser/help-output behavior.
- Add regression tests for helper functions.

Out of scope:
- CLI command redesign.
- New flags or command surface changes.

## Acceptance Criteria
- AC-1: `kamn-cli` crate exports `is_help_request` and `render_help_text`.
- AC-2: `is_help_request` returns true for `--help`/`-h` and false otherwise.
- AC-3: `render_help_text` includes usage, supported commands, and supported help flags.
- AC-4: `cargo build -p kamn-cli --release` and `cargo test -p kamn-cli` both pass.

## Conformance Cases
- C-01 (AC-1, Functional): `cargo build -p kamn-cli --release` succeeds without unresolved symbol errors.
- C-02 (AC-2, Unit): helper returns true when args include `--help`.
- C-03 (AC-2, Unit): helper returns true when args include `-h`.
- C-04 (AC-2, Unit): helper returns false when args do not include help flags.
- C-05 (AC-3, Unit): rendered help text contains `CLI_USAGE`, at least one supported command, and help flags.
- C-06 (AC-4, Functional): `cargo test -p kamn-cli` passes.

## Success Metrics
- CI `E2E Live Tests` no longer fails on unresolved `kamn_cli` help symbols.
