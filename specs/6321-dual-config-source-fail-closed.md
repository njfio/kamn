# Spec: Issue #6321 - fail closed on dual CLI/env config file declarations

## Objective

Make `kamn-node` argument layering fail closed when both config-file sources are specified
simultaneously (`--config-file` and `KAMN_NODE_CONFIG_FILE`), eliminating ambiguous source
resolution.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-node/src/cli.rs` layering functions:
    - `extract_config_file_path`
    - `build_layered_cli_args`
  - existing `parse_args` contract tests in `crates/kamn-node/src/main_tests/cli_contract_tests.rs`.
- Outputs:
  - dual-source declaration results in deterministic `ConfigError::InvalidNodeConfig`.
  - tests assert fail-closed dual-source behavior and unchanged single-source behavior.

## Boundaries/Non-goals

- In scope:
  - dual-source detection and deterministic error text.
  - tests for the specific layering behavior.
- Out of scope:
  - changing layer order (`config file -> env overrides -> CLI`).
  - adding multi-config merging.
  - changing unrelated CLI parse validation.

## Failure modes

- FM-1: both sources are set and parser silently chooses one source.
- FM-2: error type/message for dual-source ambiguity is non-deterministic.
- FM-3: single-source behavior regresses while adding dual-source guard.

## Acceptance criteria (testable booleans)

- AC-1: dual-source (`--config-file` + `KAMN_NODE_CONFIG_FILE`) returns
  `ConfigError::InvalidNodeConfig`.
- AC-2: error message is deterministic and explicitly references both source channels.
- AC-3: only `--config-file` still loads config-file args as before.
- AC-4: only `KAMN_NODE_CONFIG_FILE` still loads config-file args as before.

## Files to touch

- `specs/6321-dual-config-source-fail-closed.md`
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- `crates/kamn-node/src/cli.rs`

## Error semantics

- Fail closed using `ConfigError::InvalidNodeConfig`.
- No silent fallback when ambiguity exists.
- Preserve existing error semantics for other parse paths.

## Test plan

- RED:
  - add regression tests asserting dual-source must fail with deterministic message.
  - run targeted CLI contract lane and confirm failure.
- GREEN:
  - add dual-source guard in layering function and satisfy test expectation.
- REFACTOR:
  - keep guard logic small and self-documenting, no duplicated branching.
- INTEGRATION:
  - run targeted CLI lane plus related CLI module tests.
    - `cargo test -p kamn-node cli_contract_tests -- --nocapture`
    - `cargo test -p kamn-node cli_tests -- --nocapture`
