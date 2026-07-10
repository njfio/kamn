# Pi Persistent Live MCP Session

## Objective

Give the project-local Pi evaluator a persistent, live `kamn-mcp-server` session for Agent A so Pi can register the agent through `POST /v1/agents/register` and then query the durable profile through the same authenticated process.

## Inputs/Outputs

Inputs:

- `KAMN_MVP_LIVE_MCP_BINARY`: absolute or repository-relative `kamn-mcp-server` binary path.
- `KAMN_MVP_LIVE_MCP_ENDPOINT`: live local KAMN service endpoint.
- `KAMN_MVP_LIVE_MCP_AGENT_A_NAME`: logical Agent A name.
- `KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE`: existing private key file passed to the child process without reading it.
- Pi calls to `kamn_live_agent_a_register` and `kamn_live_agent_a_query_profile`.

Outputs:

- Registration tool result containing the service-persisted Agent A profile.
- Query tool result containing the same durable Agent A profile.
- Human-readable Pi tool messages that label the result `local-only` and make no settlement claim.

## Boundaries/Non-goals

- Preserve the existing report-derived `kamn_agent_a_register` receipt tool unchanged.
- Start `kamn-mcp-server` lazily on the first live tool call, not when the extension module loads.
- Share one child process and monotonically increasing request IDs for the Pi session.
- Pi starts neither `kamn-node` nor any devnet process.
- Never read, print, return, or persist private-key contents.
- Do not implement task invocation, Agent B, Agent C, escrow, settlement, exchange, or asset movement here.
- Do not claim generic Pi MCP-host compatibility; this is a project-local stdio bridge to KAMN's line protocol.
- Add no dependency.

## Failure Modes

- Missing or blank required environment variable: reject before spawning.
- Missing key file: reject before spawning without reading its contents.
- Child spawn failure or premature exit: reject the active call and make later calls fail rather than silently replacing identity state.
- Request timeout or abort: reject and terminate the session.
- Malformed stdout JSON: reject and terminate the session.
- Mismatched response ID: reject and terminate the session.
- MCP envelope with `ok:false`: surface the tool error and do not report success.
- Profile query before successful registration: reject without calling the server.
- Repeated shutdown: succeed without duplicate side effects.

## Acceptance Criteria

- [ ] The Pi extension registers `kamn_live_agent_a_register` and `kamn_live_agent_a_query_profile`.
- [ ] Both tools share one lazily spawned child and ordered request IDs.
- [ ] Registration dispatches `register`; query dispatches `agent_profile_query` with the registered DID.
- [ ] Required configuration and key-file existence are validated before spawn.
- [ ] The key path is passed as a child argument, while key contents remain inaccessible to tool output.
- [ ] All specified protocol, process, timeout, abort, and ordering failures are loud.
- [ ] `session_shutdown` cleanup is idempotent.
- [ ] Node contract tests prove persistence, request ordering, failures, and cleanup.
- [ ] Rust source/runbook contracts pin the live tool names and honest claim boundary.
- [ ] A real Pi run against local `kamn-node` proves register then query through the persistent process.
- [ ] Formatting, strict workspace clippy, targeted tests, and `make check` pass.

## Files To Touch

- `.pi/extensions/kamn-mvp/index.ts`
- `.pi/extensions/kamn-mvp/mcp-session.ts`
- `.pi/extensions/kamn-mvp/live-mcp-tools.ts`
- `.pi/extensions/kamn-mvp/mcp-session.test.ts`
- `.pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

All configuration and process failures throw `Error` at the Pi tool boundary. The session rejects exactly one active request, kills the child after protocol-integrity failures, and never synthesizes a fallback result. Tool-level KAMN errors retain the server's error kind and message. Error text may include binary path, endpoint, request ID, and child exit status, but never key contents.

## Test Plan

RED:

- Add Node tests that require lazy startup, same-PID reuse, request IDs `1` then `2`, DID propagation, failure handling, and idempotent shutdown.
- Extend the Rust Pi source contract to require the new tool and lifecycle markers.
- Extend the runbook contract to require configuration, local-only labeling, and the non-settlement boundary.

GREEN:

- Implement the minimum persistent line-protocol session and two Pi tools.
- Document the exact evaluator commands.

REFACTOR:

- Separate process/protocol ownership from Pi tool registration.
- Verify functions remain single-purpose and files remain within repository size limits.

INTEGRATION:

- Build real `kamn-node` and `kamn-mcp-server` binaries.
- Start a disposable loopback node and key file.
- Run Pi with Codex auth and only the two live tools.
- Prove the node receives registration nonce 1 and profile query nonce 2 for the same DID.
- Run targeted contracts, `cargo fmt --check`, strict clippy, and `make check`.
