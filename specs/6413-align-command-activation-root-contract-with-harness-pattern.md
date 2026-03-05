# Spec: Issue #6413 - Align command activation root contract with harness pattern

## Objective

Refactor `crates/kamn-cli/tests/command_activation_contract.rs` to use the shared `with_contract_server` test harness for server-backed health contract paths, eliminating duplicated manual server setup while preserving existing command assertions and behavior.

## Inputs/Outputs

- Inputs:
  - root contract file: `crates/kamn-cli/tests/command_activation_contract.rs`
  - shared harness: `crates/kamn-cli/tests/command_activation_harness.rs`
  - follow-up regression guard: `crates/kamn-cli/tests/command_activation_followup_contract.rs`
- Outputs:
  - root health tests wired through `with_contract_server`
  - follow-up contract guard that fails if manual root server setup markers return
  - passing root + follow-up contract test lanes

## Boundaries/Non-goals

- In scope:
  - replacing root manual server thread/bootstrap in health contract tests
  - enforcing harness pattern via follow-up contract checks
- Out of scope:
  - changing CLI command semantics or payload contracts
  - changing production CLI code
  - changing non-root command activation contract files

## Failure modes

- FM-1: root contract file continues using manual server bootstrap markers.
- FM-2: health command tests no longer run against a real request-serving path.
- FM-3: follow-up contract does not guard harness usage, allowing regression.
- FM-4: root/follow-up contract suites fail.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `spec_c01_cli_health_command_executes_supported_path` and `spec_c06_cli_json_output_contract_renders_structured_health_projection` use `with_contract_server`.
- [ ] AC-2: root contract file removes direct `reserve_loopback_addr`/`run_cli_contract_server`/`wait_for_server_ready` + `thread::spawn` setup in health tests.
- [ ] AC-3: follow-up contract contains a guard that root contract uses `with_contract_server` and rejects manual bootstrap markers.
- [ ] AC-4: `cargo test -p kamn-cli --test command_activation_followup_contract` passes.
- [ ] AC-5: `cargo test -p kamn-cli --test command_activation_contract` passes.

## Files to touch

- `specs/6413-align-command-activation-root-contract-with-harness-pattern.md`
- `crates/kamn-cli/tests/command_activation_followup_contract.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`

## Error semantics

- No runtime error-semantic change. This is test-only harness wiring.
- Fail-closed behavior is preserved through assertion failures in contract tests.

## Test plan

- RED:
  - add follow-up contract assertion requiring `with_contract_server` marker in root file and banning manual bootstrap markers.
  - verify follow-up contract fails before root refactor.
- GREEN:
  - refactor root health tests to use shared harness helper.
  - rerun follow-up and root contract lanes.
- REFACTOR:
  - tighten imports/duplication and preserve readability.
- INTEGRATION:
  - run both contract suites and record results in spec.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
