# Spec: Issue 6377 - Harden command activation split follow-ups

## Objective
Address audit-reported quality gaps in the `#6373` command activation split by centralizing duplicated test helpers, restoring assertion diagnostics, correcting historical spec deviations, and documenting dead-code allowances.

## Inputs/Outputs
- Inputs:
  - Existing split command activation test files and harness introduced by `#6373`.
- Outputs:
  - Shared helper ownership consolidated in `command_activation_harness.rs`.
  - Core/content contract tests using shared helper APIs without local duplication.
  - Restored assertion failure messages for moved tests.
  - Updated `#6373` spec deviations to reflect extra files introduced.

## Boundaries/Non-goals
- Do not change CLI runtime behavior.
- Do not add dependencies.
- Do not alter command parsing semantics.

## Failure modes
- Helper refactor accidentally changes test behavior.
- Assertion message updates miss key contract assertions.
- Spec correction remains inconsistent with repository history.

## Acceptance criteria (testable booleans)
- [x] Shared test helpers are centralized so core/content contract files no longer duplicate `with_contract_server` and missing-arg assertions.
- [x] Key assertions in moved command activation tests include explicit diagnostic messages for CI triage.
- [x] `specs/6373-kamn-cli-command-activation-contract-split.md` deviations section reflects the actual extra files introduced.
- [x] `#[allow(dead_code)]` usage in activation harness includes an explanatory comment.
- [x] `kamn-cli` activation contract test suite remains green.

## Files to touch
- `specs/6377-command-activation-followup-hardening.md`
- `specs/6373-kamn-cli-command-activation-contract-split.md`
- `crates/kamn-cli/tests/command_activation_harness.rs`
- `crates/kamn-cli/tests/command_activation_core_contract.rs`
- `crates/kamn-cli/tests/command_activation_core_contract/cases.rs` (new)
- `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs`
- `crates/kamn-cli/tests/command_activation_followup_contract.rs` (new)

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Add source-level follow-up contract tests asserting helper centralization, diagnostic message presence, spec deviation accuracy, and dead-code rationale comments.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test command_activation_followup_contract`
  - `cargo test -p kamn-cli --test command_activation_core_contract`
  - `cargo test -p kamn-cli --test command_activation_content_bridge_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Phase 6 integration evidence
- Wiring:
  - Centralized `with_contract_server` and invalid-input helper assertions in `command_activation_harness.rs`.
  - Removed duplicate helper implementations from core/content contract files.
  - Restored explicit assertion diagnostic labels in moved command activation suites.
  - Corrected `#6373` spec deviations to include extra files introduced by the split.
- Executed:
  - `cargo test -p kamn-cli --test command_activation_followup_contract`
  - `cargo test -p kamn-cli --test command_activation_split_contract`
  - `cargo test -p kamn-cli --test command_activation_core_contract`
  - `cargo test -p kamn-cli --test command_activation_content_bridge_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`

## Deviations
- None.
