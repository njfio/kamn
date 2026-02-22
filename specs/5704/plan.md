# Plan: #5704 Align `kamn-cli` Default/Output Contract with PRD JSON Semantics

## Approach
1. Add RED tests for parser default and renderer behavior (JSON default, JSON structure, text compatibility).
2. Introduce a typed CLI command output model that carries both JSON and text projections.
3. Update command modules to return structured output payloads with deterministic key names.
4. Update main rendering path to emit the selected projection directly (`json` or `text`) without wrapping blobs.
5. Run full CLI regression/lint/format checks.

## Affected Modules
- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/src/main.rs`
- `crates/kamn-cli/src/commands/*.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`
- `crates/kamn-cli/tests/subcommand_surface_contract.rs`

## Risks and Mitigations
- Risk: changing dispatch return type may break command tests broadly.
  Mitigation: introduce small output struct with helper constructors and update tests in one slice.

- Risk: JSON field naming drift across commands.
  Mitigation: codify deterministic field names in command modules and assert in contract tests.

## Interfaces / Contracts
- `parse_cli_args` default format changes to `OutputFormat::Json`.
- `dispatch` return contract changes from `String` to typed output projection carrier.
- CLI main prints selected output projection directly.

## ADR
- Not required: no dependency/protocol/wire-format change outside CLI output surface.
