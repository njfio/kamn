# Spec: Issue #6415 - Align content-bridge activation contract factoring with core cases pattern

## Objective

Refactor `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs` so scenario bodies are delegated to a dedicated `cases.rs` module, matching the factoring pattern already used by `command_activation_core_contract.rs`, while preserving existing assertions and command behavior.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs`
  - `crates/kamn-cli/tests/command_activation_harness.rs`
  - `crates/kamn-cli/tests/command_activation_followup_contract.rs`
- Outputs:
  - new `crates/kamn-cli/tests/command_activation_content_bridge_contract/cases.rs`
  - root content-bridge contract file delegates to cases module entrypoints
  - follow-up guard that enforces delegated factoring markers

## Boundaries/Non-goals

- In scope:
  - test-module factoring only for content/bridge command activation contract coverage
  - keeping assertion diagnostics explicit and deterministic
- Out of scope:
  - modifying production CLI code
  - changing command payloads, endpoints, or response semantics
  - moving/rewriting core command activation contract factoring

## Failure modes

- FM-1: content-bridge scenarios remain inline and do not delegate to a cases module.
- FM-2: assertion diagnostic labels regress during refactor.
- FM-3: no regression guard exists for the factoring layout.
- FM-4: follow-up or content-bridge contract suites fail.

## Acceptance criteria (testable booleans)

- [ ] AC-1: `tests/command_activation_content_bridge_contract/cases.rs` exists and owns spec C08/C09 scenario bodies.
- [ ] AC-2: `command_activation_content_bridge_contract.rs` delegates C08/C09 test execution to the cases module.
- [ ] AC-3: follow-up contract contains checks for content-bridge factoring markers (delegate calls + cases module markers).
- [ ] AC-4: `cargo test -p kamn-cli --test command_activation_followup_contract` passes.
- [ ] AC-5: `cargo test -p kamn-cli --test command_activation_content_bridge_contract` passes.

## Files to touch

- `specs/6415-content-bridge-contract-cases-factoring.md`
- `crates/kamn-cli/tests/command_activation_content_bridge_contract.rs`
- `crates/kamn-cli/tests/command_activation_content_bridge_contract/cases.rs`
- `crates/kamn-cli/tests/command_activation_followup_contract.rs`

## Error semantics

- No runtime error-semantic changes; tests continue to fail closed via assertions.

## Test plan

- RED:
  - add follow-up contract checks that require content-bridge delegation/cases markers.
  - confirm follow-up contract fails before refactor.
- GREEN:
  - introduce content-bridge `cases.rs` and delegate C08/C09 entrypoint tests.
  - keep diagnostics via constants in root file and assertions in cases module.
- REFACTOR:
  - remove any duplicated assertion scaffolding and keep imports/module boundaries concise.
- INTEGRATION:
  - run follow-up contract suite and content-bridge contract suite; record pass evidence.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
