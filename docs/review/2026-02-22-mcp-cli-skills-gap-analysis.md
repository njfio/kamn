# MCP and CLI Skills Functionality for Agents — Gap Analysis

**Date:** 2026-02-22
**Scope:** Evaluate MCP (Model Context Protocol) and CLI skills functionality for KAMN agents, identify gaps.

## 1. Current State of MCP Integration

### What Exists

The MCP integration is currently a **mapping-only / conformance-only layer**, not a runtime implementation:

- **`docs/foundation/a2a-mcp-interoperability.md`** — Static mapping document defining how KAMN envelope types and task states correspond to A2A and MCP concepts:
  - `Request` → `task.invoke` / `tool_call`
  - `Response` → `task.result` / `tool_result`
  - `Event` → `event.notify` / `notification`
  - Task states: `Submitted→pending`, `InProgress→running`, `Completed→succeeded`, etc.

- **Conformance harness** (`scripts/message/run_a2a_mcp_conformance_harness.py`) — Python fixture replay engine validating mapping correctness against 4 test cases in `fixtures/a2a_mcp_conformance/replay_cases.json`. This is a CI policy gate, not a runtime translation layer.

- **Docs-contract tests** (`crates/kamn-core/tests/docs_contract_wave4_harness.rs`) — Rust tests asserting the A2A/MCP doc contains expected mapping tables, SDK examples, and conformance harness references.

### MCP Gaps

| Gap | Severity | Description |
|---|---|---|
| No MCP server implementation | High | No `mcp_server.rs`, no stdio/SSE/HTTP MCP transport, no JSON-RPC 2.0 handler. Agents cannot expose tools via MCP. |
| No MCP tool registry | High | No `tools/list` or `tools/call` handler. No way for external clients to discover or invoke KAMN agent capabilities as MCP tools. |
| No MCP client implementation | High | Agents cannot connect to external MCP servers to discover and invoke external tools. |
| No runtime type translation | Medium | A2A/MCP mapping doc is purely conceptual. No Rust code converts a `CanonicalMessageEnvelope` with `Request` type into an actual MCP `tool_call` JSON-RPC message. |
| No MCP resource/prompt primitives | Medium | MCP supports `resources/list`, `resources/read`, `prompts/list`, `prompts/get` — none mapped or implemented. |
| No streaming support | Medium | MCP supports SSE for streaming tool results — no SSE transport exists. |
| Conformance fixture is minimal | Low | Only 4 cases: 3 happy-path mappings + 1 mismatch. Missing negative cases for unknown types, partial mappings, edge cases. |

## 2. Current State of CLI Skills for Agents

### What Exists

The CLI (`kamn-node`) is an infrastructure-focused node binary, not an agent skills interface:

- **`crates/kamn-node/src/cli.rs`** — Hand-rolled argument parser for node configuration (role, chain-id, storage-dir, daemon settings, Kolme live settings, API bind addresses, observability endpoints). Config layering: CLI args > env vars > config file.
- No subcommands, no agent skill dispatch, no interactive agent commands.

### CLI Skills Gaps

| Gap | Severity | Description |
|---|---|---|
| No agent-facing CLI subcommands | High | No `kamn agent register`, `kamn agent send`, `kamn task create`, `kamn task status`, `kamn skill invoke`. CLI is purely for node operators. |
| No skill registry or dispatch | High | Zero mentions of "skill" in Rust code. No `SkillRegistry`, `SkillDefinition`, or skill execution framework. |
| No CLI → Service API bridge | Medium | `ServiceApiClient` has all HTTP routes but no CLI wrapper. Agents must use Rust/TS SDK programmatically. |
| No interactive REPL or agent shell | Low | No interactive mode for ad-hoc agent operations. |

## 3. SDK Agent Layer Assessment

### What Exists and Works Well

- **`KamnAgent` trait** (`crates/kamn-sdk/src/agent.rs`) — 13 methods: `register`, `resolve`, `send`, `receive`, `receive_stream`, `create_task`, `accept_task`, `submit_artifact`, `complete_task`, `create_escrow`, `release_escrow`, `balance`, `search_agents`, `get_reputation`.
- **Transport modes** — `InMemory` (deterministic tests) and `Live` (endpoint-scoped shared state).
- **`ServiceApiClient`** (`crates/kamn-sdk/src/service.rs`) — Synchronous HTTP client with DID-authn, all `/v1/` routes, WebSocket events.
- **`TcpTransportAdapter`** (`crates/kamn-sdk/src/tcp.rs`) — Wire-level signed envelope transport.
- **TypeScript SDK** (`packages/kamn-sdk/`) — Parity with Rust SDK for memory/live clients + `OpenClawConnector`.
- **Service API** (`crates/kamn-node/src/service_api_endpoint/`) — Full axum-based HTTP/WS server with auth middleware, anti-spam, rate limiting, TLS, presence tracking.

### SDK Gaps Relevant to MCP/Skills

| Gap | Severity | Description |
|---|---|---|
| No tool/capability advertisement | High | `AgentMetadata.capabilities` is `Vec<String>` of labels. No structured tool definition (input schema, output schema, description). Cannot bridge to MCP `tools/list`. |
| No capability invocation protocol | High | No way to invoke a specific capability by name with typed parameters. `send()` takes a string body — invocation semantics are opaque. |
| No tool result streaming | Medium | `receive()` drains all messages atomically. No incremental/streaming result delivery aligned with MCP patterns. |
| HTTPS not supported in ServiceApiClient | Low | `connect_stream()` returns `SdkError::NotImplemented` for HTTPS. Only plaintext HTTP works. |

## 4. End-to-End Architecture Gap Map

```
External AI Client (Claude, Cursor, etc.)
    │
    ▼
┌─────────────────────────────────────┐
│  MCP Server (JSON-RPC 2.0)          │  ← MISSING: No MCP server
│  ├─ tools/list → skill registry     │  ← MISSING: No tool/skill registry
│  ├─ tools/call → skill dispatch     │  ← MISSING: No skill execution
│  ├─ resources/list                  │  ← MISSING: No resource adapter
│  └─ stdio / SSE / HTTP transport    │  ← MISSING: No MCP transport
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  CLI Skills Layer                   │  ← MISSING: No CLI subcommands
│  ├─ kamn agent register/send        │    for agent operations
│  ├─ kamn task create/status         │
│  └─ kamn skill invoke               │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  SDK Agent Layer (exists)           │  OK: KamnAgent trait, transport modes
│  ├─ KamnAgent trait (13 methods)    │  OK: Service API client
│  ├─ ServiceApiClient (HTTP/WS)      │  GAP: No structured tool definitions
│  └─ TcpTransportAdapter             │  GAP: No capability invocation protocol
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Service API (exists)               │  OK: Full HTTP/WS server with auth
│  ├─ /v1/messages/send               │  OK: Anti-spam, rate limiting, TLS
│  ├─ /v1/tasks/create                │  OK: WebSocket events + presence
│  └─ /v1/events/ws                   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  Core Runtime (exists)              │  OK: Message lifecycle, data layers
│  ├─ Message envelope + crypto       │  OK: Kolme blockchain anchoring
│  ├─ Data layers M0-M11             │  OK: Snapshot persistence
│  └─ Delivery guards + proofs        │  OK: Extensive conformance testing
└─────────────────────────────────────┘
```

## 5. Prioritized Recommendations

### P0 — Critical Path

1. **MCP Server with Tool Registry** — Implement an MCP server wrapping existing `ServiceApiClient` routes as discoverable tools. Start with stdio transport + `tools/list` + `tools/call`. This enables any MCP-compatible AI client to interact with KAMN agents.

2. **Structured Skill/Tool Definitions** — Extend `AgentMetadata` (or add a parallel `ToolDefinition` struct) with JSON Schema input/output descriptions so capabilities can be advertised via MCP `tools/list`.

### P1 — High Value

3. **CLI Agent Subcommands** — Add `kamn agent` / `kamn task` / `kamn message` subcommand tree wrapping `ServiceApiClient` for operator/developer use. Enables both human and automated workflows.

4. **Runtime Type Translation** — Implement Rust translation layer between `CanonicalMessageEnvelope` and MCP JSON-RPC messages, moving beyond the static mapping doc.

### P2 — Quality and Completeness

5. **Expand A2A/MCP Conformance Fixtures** — Add negative/edge cases: unknown message types, partial lifecycle mappings, schema drift scenarios.

6. **HTTPS Support in ServiceApiClient** — Enable TLS for the SDK client to match server-side TLS capability.
