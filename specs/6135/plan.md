# Plan: Issue #6135

## Approach
1. Add a lightweight JSON boolean helper (`json_optional_bool_field`) alongside existing string/u64 helpers in `mcp_agent.rs`.
2. Replace success checks in `run_live_s04_mcp_tool_call` and `validate_probe_health_response` with boolean helper evaluation.
3. Expand tests:
   - helper-level extraction behavior;
   - health validation failure modes;
   - live tool-call probe rejecting non-boolean `ok` values.
4. Run targeted harness tests and lint for the crate.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/*` (if needed; primary coverage stays in unit tests in driver module)

## Risks / Mitigations
- Risk: helper parser may still misread malformed JSON-like text.
  Mitigation: require exact token start after key marker and explicit `true`/`false` token boundary checks.
- Risk: behavior drift in existing probe flow.
  Mitigation: preserve framed parsing flow and extend existing `run_live_s04_mcp_tool_call` tests for green-path continuity.

## Interfaces / Contracts
- No public API changes.
- Internal contract change: probe success is now `ok == true` as parsed boolean.
