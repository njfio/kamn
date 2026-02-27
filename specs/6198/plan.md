# Plan: Issue 6198 - CLI Help/Usage Contract Surface

- Issue: #6198
- Milestone: `R59 Swarm Gap Closure`

## Approach

1. Add `Help` to `CommandKind` and parse aliases: `help`, `--help`, `-h`.
2. Add deterministic usage metadata constants.
3. Add `dispatch` branch returning synthetic help payload (`CommandOutput`).
4. Add unit regressions for parser + dispatch behavior.
5. Run existing command activation integration tests to ensure no regression.

## Affected Modules

- `crates/kamn-cli/src/lib.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`

## Risks and Mitigations

1. Risk: unknown flags accidentally parsed as passthrough for non-help commands.
   - Mitigation: scope change only to explicit help tokens.
2. Risk: command list drift.
   - Mitigation: centralize usage/help constants in one module.
