# Plan: #5670 Activate `kamn-cli` Execution for Supported Operations

## Approach
1. Add RED tests for `health`, `list-messages`, and `verify-proof` command contracts.
2. Implement shared command helper to construct `KamnAgentHandle` from CLI args + env defaults.
3. Implement supported command modules with deterministic argument validation.
4. Keep unsupported command modules unchanged and add regression assertions.
5. Run full `kamn-cli` test/lint/format gates.

## Affected Modules
- `crates/kamn-cli/src/commands/mod.rs`
- `crates/kamn-cli/src/commands/health.rs`
- `crates/kamn-cli/src/commands/list_messages.rs`
- `crates/kamn-cli/src/commands/verify_proof.rs`
- `crates/kamn-cli/tests/*` (new)

## Risks and Mitigations
- Risk: test harness complexity for live service dependencies.
- Mitigation: use lightweight local contract server for health/list-messages and keep proof verification deterministic/local.
- Risk: positional CLI args can drift in semantics.
- Mitigation: explicit required-arg helpers and deterministic invalid-input messages.

## Interfaces / Contracts
- New helper: bootstrap `KamnAgentHandle` with env defaults (`KAMN_AGENT_NAME`, `KAMN_KOLME_ENDPOINT`).
- Command contracts:
  - `health`: no required passthrough args.
  - `list-messages`: requires `channel_id` at passthrough index 0.
  - `verify-proof`: requires 4 args (`message_id`, `tx_hash`, `block_height`, `finality`).

## ADR
- Not required. No architecture/dependency changes.
