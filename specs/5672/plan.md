# Plan: #5672 Activate Remaining CLI Core Message/Task Operations

## Approach
1. Add RED tests for each command's success + missing-arg failure contract.
2. Implement command modules using existing shared helper (`connect_handle`, `required_arg`).
3. Keep unsupported command modules unchanged and validate regression behavior.
4. Run full `kamn-cli` test/lint/format gates.

## Affected Modules
- `crates/kamn-cli/src/commands/register.rs`
- `crates/kamn-cli/src/commands/send_message.rs`
- `crates/kamn-cli/src/commands/create_channel.rs`
- `crates/kamn-cli/src/commands/query_message.rs`
- `crates/kamn-cli/src/commands/create_task.rs`
- `crates/kamn-cli/tests/command_activation_contract.rs`

## Risks and Mitigations
- Risk: command arg ordering ambiguity.
- Mitigation: strict positional-arg contract with explicit invalid-input errors.
- Risk: local contract server mismatch with expected payload fields.
- Mitigation: deterministic mock responses aligned to SDK/agent-lib receipt shapes.

## Interfaces / Contracts
- `register`: no required args.
- `send-message`: payload arg at passthrough index 0.
- `create-channel`: payload arg at passthrough index 0.
- `query-message`: message_id arg at passthrough index 0.
- `create-task`: payload arg at passthrough index 0.

## ADR
- Not required.
