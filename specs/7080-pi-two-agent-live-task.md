# Pi Two-Agent Live Task

## Objective

Let Pi drive one real local-only task lifecycle through two independently authenticated KAMN agents: Agent A and Agent B register through separate persistent `kamn-mcp-server` children, Agent A creates a task, Agent B accepts it, and both query the same accepted task state.

## Inputs/Outputs

Inputs:

- Shared `KAMN_MVP_LIVE_MCP_BINARY` and `KAMN_MVP_LIVE_MCP_ENDPOINT`.
- Agent A `KAMN_MVP_LIVE_MCP_AGENT_A_NAME` and `KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE`.
- Agent B `KAMN_MVP_LIVE_MCP_AGENT_B_NAME` and `KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE`.
- Non-empty task title and description supplied to `kamn_live_agent_a_create_task`.
- Ordered Pi calls to register, create, accept, and query tools.

Outputs:

- Two distinct service-persisted agent DIDs.
- One non-empty service task ID.
- `submitted` creation state followed by `accepted` acceptance state.
- Matching accepted task projections from Agent A and Agent B queries.
- Tool messages and details labelled `real local-only task lifecycle`.

## Boundaries/Non-goals

- Reuse the existing line-protocol `McpSession`; do not add another transport or dependency.
- Preserve report-derived actor receipt tools unchanged.
- Preserve existing live Agent A register/profile behavior.
- Keep one child and request sequence per agent for the Pi session.
- Never expose key contents or forward Pi/OpenAI credentials.
- Do not fund/release escrow or claim settlement, exchange, asset movement, Solana devnet, Agent C verification, MCP-child restart durability, or node-restart durability.

## Failure Modes

- Missing Agent B name/key configuration or key file: reject before spawning B.
- Equal Agent A and Agent B DIDs: reject the second registration result.
- Create before Agent A registration: reject without MCP dispatch.
- Accept before Agent B registration or task creation: reject without MCP dispatch.
- Query before the matching agent registration or task creation: reject without MCP dispatch.
- Blank task title/description: reject without MCP dispatch.
- Create result missing task ID or state: reject loudly.
- Accept/query result task ID differs from stored task ID: reject loudly.
- Create state is not `submitted`, or accept/final query state is not `accepted`: reject loudly.
- Either child/process/protocol/auth call fails: surface the existing hard failure with no fallback.
- Session shutdown: terminate both children idempotently.

## Acceptance Criteria

- [ ] `kamn_live_agent_b_register` uses Agent B configuration and a distinct persistent child.
- [ ] Agent A and Agent B registration DIDs are non-empty and distinct.
- [ ] `kamn_live_agent_a_create_task` JSON-encodes validated title/description and dispatches `create_task` through Agent A.
- [ ] `kamn_live_agent_b_accept_task` dispatches `accept_task` with the stored task ID through Agent B.
- [ ] `kamn_live_agent_a_query_task` and `kamn_live_agent_b_query_task` dispatch `query_task` through their respective sessions.
- [ ] Both query tools require and return the same task ID with state `accepted`.
- [ ] Agent A request IDs progress independently from Agent B request IDs.
- [ ] Both child processes are cleaned up on Pi shutdown.
- [ ] Automated tests cover ordered success and every specified prerequisite/result failure.
- [ ] Rust source/runbook contracts pin the tool names, two-agent configuration, and local-only/non-settlement boundary.
- [ ] A real Pi run proves two distinct DIDs, A register/create/query nonces `1/2/3`, B register/accept/query nonces `1/2/3`, and matching accepted state.
- [ ] Formatting, strict clippy, targeted contracts, `make check`, `make demo-mvp`, and the canonical verifier pass.

## Files To Touch

- `.pi/extensions/kamn-mvp/mcp-session.ts`
- `.pi/extensions/kamn-mvp/mcp-session.test.ts`
- `.pi/extensions/kamn-mvp/live-mcp-tools.ts`
- `.pi/extensions/kamn-mvp/live-mcp-tools.test.ts`
- `.pi/extensions/kamn-mvp/test-fixtures/fake-mcp-server.mjs`
- `crates/kamn-e2e-harness/tests/mvp_demo_agent_harness_claim_contract.rs`
- `crates/kamn-e2e-harness/tests/mvp_evaluator_demo_runbook_contract.rs`
- `docs/validation/mvp-evaluator-demo.md`

## Error Semantics

Pi tool-order and result-validation failures throw `Error` before reporting success. Existing `McpSession` process/protocol errors remain terminal to only the affected agent session. No agent session is replaced automatically, no task ID is synthesized, and no query result is coerced to the expected state.

## Test Plan

RED:

- Extend session tests to require Agent B configuration isolation.
- Add live-tool tests with the process-boundary fake server for two distinct child PIDs, independent request sequences, exact task payload/ID propagation, ordering failures, and dual shutdown.
- Extend Rust source and runbook contracts with the new tools and claim boundary.

GREEN:

- Generalize configuration parsing by agent role.
- Add Agent B registration and Agent A/B task tools with shared task state.
- Extend the evaluator runbook.

REFACTOR:

- Keep process ownership in `mcp-session.ts` and workflow state/tool registration in focused modules under file/function limits.
- Remove duplication without generalizing beyond two named MVP agents.

INTEGRATION:

- Build real binaries and start one disposable local node with durable storage.
- Run Pi with Codex OAuth and only the six live identity/task tools.
- Inspect node logs for distinct DIDs and independent nonce sequences.
- Run Node tests, evaluator contracts, formatting, strict clippy, `make check`, canonical demo, and verifier.
