# Spec: Issue 6373 - Split kamn-cli `command_activation_contract` test surface

## Objective
Reduce `crates/kamn-cli/tests/command_activation_contract.rs` complexity by splitting command-family assertions into focused integration test files with shared harness helpers.

## Inputs/Outputs
- Inputs:
  - Existing monolithic `command_activation_contract.rs` test file (~800 LOC).
- Outputs:
  - Shared activation harness helper module for request server + parsed argument builders.
  - Focused contract files by command family (help/health, messaging/task, content/bridge, etc.).
  - Reduced root activation contract file size and clearer test ownership boundaries.

## Boundaries/Non-goals
- Do not change runtime command behavior, parsing semantics, outputs, or error text.
- Do not remove assertions without equivalent replacement.
- Do not add dependencies.

## Failure modes
- Coverage regression if assertions are dropped during split.
- Shared harness drift introduces flaky request handling.
- New files remain oversized and fail maintainability intent.

## Acceptance criteria (testable booleans)
- [x] Command activation contracts are split into focused test files by command family.
- [x] Shared harness helpers are centralized in dedicated helper module(s).
- [x] Existing assertion behaviors are preserved with equivalent coverage.
- [x] Command activation suites and related `kamn-cli` contract tests remain green.

## Files to touch
- `specs/6373-kamn-cli-command-activation-contract-split.md`
- `crates/kamn-cli/tests/command_activation_contract.rs`
- `crates/kamn-cli/tests/command_activation_harness.rs` (new)
- `crates/kamn-cli/tests/command_activation_core_contract.rs` (new)
- `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs` (new)

## Error semantics
- No runtime error behavior changes.
- No silent fallback behavior introduced.

## Test plan
- Red:
  - Add split-target contract tests asserting size/ownership boundaries and fail until split is applied.
- Green/Refactor/Integration:
  - `cargo test -p kamn-cli --test command_activation_contract`
  - `cargo test -p kamn-cli --test command_activation_core_contract`
  - `cargo test -p kamn-cli --test command_activation_content_bridge_contract`
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`

## Phase 6 integration evidence
- Split wiring:
  - `command_activation_contract.rs` now owns health/help/json assertions only.
  - `command_activation_core_contract.rs` owns messaging/task/profile assertions.
  - `command_activation_content_bridge_contract.rs` owns content/bridge assertions.
  - `command_activation_harness.rs` centralizes shared dispatch/server argument helpers.
- Executed:
  - `cargo test -p kamn-cli --test command_activation_split_contract`
  - `cargo test -p kamn-cli --test command_activation_contract`
  - `cargo test -p kamn-cli --test command_activation_core_contract`
  - `cargo test -p kamn-cli --test command_activation_content_bridge_contract`
  - `cargo test -p kamn-cli --test main_contract`
  - `cargo test -p kamn-cli --test subcommand_surface_contract`

## Deviations
- Introduced `crates/kamn-cli/tests/command_activation_harness_routes.rs` to keep harness logic split within file-size constraints.
- Introduced `crates/kamn-cli/tests/command_activation_split_contract.rs` as a source-level ownership guard for the split migration.
