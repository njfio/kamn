# Issue 7117: Align README Front-Door Contract

## Objective

Align the README front-door test with the compact evaluator headings merged by
#7111 without weakening any product, proof, or claim-boundary requirement.

## Inputs And Outputs

Input: The current root `README.md` and its front-door contract test.

Output: A green contract that enforces the current human-first section names and
ordering.

## Boundaries And Non-Goals

- Update test heading literals and ordering only.
- Do not rewrite README copy or remove required content markers.
- Do not change commands, claim semantics, or production code.

## Failure Modes

- The test still requires superseded headings.
- Required demo, verifier, devnet, runbook, or claim-boundary text is removed.
- AI-maintainer guidance appears before the human evaluator front door.

## Acceptance Criteria

- [x] Current headings are required in their documented order.
- [x] All non-heading MVP markers remain required.
- [x] The focused README contract passes.
- [x] Formatting and strict targeted clippy pass.

## Files To Touch

- `crates/kamn-e2e-harness/tests/readme_mvp_front_door_contract.rs`
- This spec only.

## Error Semantics

Test failures continue to identify the missing heading or ordering pair.

## Test Plan

RED is the current failure for missing `## What KAMN Proves Today`.

GREEN:

```bash
cargo test -p kamn-e2e-harness --test readme_mvp_front_door_contract
cargo clippy -p kamn-e2e-harness \
  --test readme_mvp_front_door_contract -- -D warnings
```
