# MCP Server and CLI Skills Implementation Roadmap

**Date:** 2026-02-22
**Prerequisite:** [Gap Analysis](../review/2026-02-22-mcp-cli-skills-gap-analysis.md)
**Status:** Draft

This document specifies exactly what needs to be built to give KAMN agents MCP server/client capabilities, a CLI skills surface, and structured tool definitions. It is organized into four phases with concrete module boundaries, struct/trait signatures, wire formats, and acceptance criteria.

---

## Phase 1 — Tool Definition Model and Registry

**Goal:** Introduce a structured, schema-aware tool definition that replaces the opaque `Vec<String>` capabilities on `AgentMetadata`, and a registry that agents can populate and query.

### 1.1 New Types (`kamn-core`)

**File:** `crates/kamn-core/src/tool_definition.rs`

```rust
/// JSON-Schema-lite description of a tool's input or output shape.
pub struct ToolParameterSchema {
    /// Parameter name.
    pub name: String,
    /// JSON Schema type: "string", "number", "boolean", "object", "array".
    pub schema_type: String,
    /// Human-readable description of the parameter.
    pub description: String,
    /// Whether the parameter is required.
    pub required: bool,
}

/// A single tool that an agent can advertise and execute.
pub struct ToolDefinition {
    /// Stable tool identifier, e.g. "kamn:tool:send-message:v1".
    pub tool_id: String,
    /// Human-readable tool name shown in tool listings.
    pub display_name: String,
    /// One-paragraph description of what the tool does.
    pub description: String,
    /// Ordered list of input parameters with schema metadata.
    pub input_schema: Vec<ToolParameterSchema>,
    /// Ordered list of output fields with schema metadata.
    pub output_schema: Vec<ToolParameterSchema>,
    /// Agent DID that owns/provides this tool (set at registration time).
    pub provider_did: String,
}

/// Validation errors for tool definitions.
pub enum ToolDefinitionError {
    EmptyToolId,
    EmptyDisplayName,
    EmptyDescription,
    EmptyParameterName { index: usize },
    InvalidSchemaType { index: usize, value: String },
    DuplicateParameterName { name: String },
}
```

**Validation contract:**
- `tool_id` must be non-empty, must match `kamn:tool:<name>:v<N>` format.
- `display_name` and `description` must be non-empty trimmed strings.
- Each parameter name must be unique within its schema list.
- `schema_type` must be one of the 5 allowed JSON Schema types.
- `provider_did` must parse as a valid `kamn:did:agent:*` identifier.

### 1.2 Tool Registry (`kamn-core`)

**File:** `crates/kamn-core/src/tool_registry.rs`

```rust
/// In-memory registry of tools advertised by agents.
pub struct ToolRegistry {
    /// Tools indexed by tool_id.
    tools_by_id: BTreeMap<String, ToolDefinition>,
    /// Tool IDs indexed by provider DID.
    tools_by_provider: BTreeMap<String, BTreeSet<String>>,
}

impl ToolRegistry {
    pub fn new() -> Self;

    /// Registers a validated tool definition. Rejects duplicates.
    pub fn register(&mut self, tool: ToolDefinition) -> Result<(), ToolRegistryError>;

    /// Removes a tool by ID. Returns error if not found.
    pub fn deregister(&mut self, tool_id: &str) -> Result<(), ToolRegistryError>;

    /// Lists all registered tools (deterministic order by tool_id).
    pub fn list_tools(&self) -> Vec<&ToolDefinition>;

    /// Lists tools for a specific provider DID.
    pub fn list_tools_by_provider(&self, provider_did: &str) -> Vec<&ToolDefinition>;

    /// Looks up a single tool by ID.
    pub fn get_tool(&self, tool_id: &str) -> Option<&ToolDefinition>;

    /// Snapshot export for persistence.
    pub fn export_snapshot(&self) -> ToolRegistrySnapshot;

    /// Snapshot restore with validation.
    pub fn restore_snapshot(snapshot: ToolRegistrySnapshot) -> Result<Self, ToolRegistryError>;
}
```

### 1.3 Built-in Tool Definitions

Register canonical tools that map to existing `KamnAgent` trait methods. These are the "default skills" every KAMN node exposes:

| Tool ID | Maps To | Input | Output |
|---|---|---|---|
| `kamn:tool:agent-register:v1` | `KamnAgent::register()` | agent_type, model_family, capabilities | did |
| `kamn:tool:agent-resolve:v1` | `KamnAgent::resolve()` | did | did_document |
| `kamn:tool:message-send:v1` | `KamnAgent::send()` | from, to, body, channel? | message_id |
| `kamn:tool:message-receive:v1` | `KamnAgent::receive()` | did | messages[] |
| `kamn:tool:task-create:v1` | `KamnAgent::create_task()` | creator, task_type, description | task_id |
| `kamn:tool:task-accept:v1` | `KamnAgent::accept_task()` | task_id, assignee | (empty) |
| `kamn:tool:task-complete:v1` | `KamnAgent::complete_task()` | task_id | (empty) |
| `kamn:tool:artifact-submit:v1` | `KamnAgent::submit_artifact()` | task_id, name, bytes_base64 | artifact_id |
| `kamn:tool:escrow-create:v1` | `KamnAgent::create_escrow()` | payer, payee, amount | escrow_id |
| `kamn:tool:escrow-release:v1` | `KamnAgent::release_escrow()` | escrow_id | (empty) |
| `kamn:tool:balance-query:v1` | `KamnAgent::balance()` | did | amount |
| `kamn:tool:agent-search:v1` | `KamnAgent::search_agents()` | capability?, model_family? | agents[] |
| `kamn:tool:reputation-query:v1` | `KamnAgent::get_reputation()` | did | score |

**File:** `crates/kamn-core/src/tool_builtins.rs` — function `register_builtin_tools(registry: &mut ToolRegistry)` that populates all 13 tool definitions.

### 1.4 SDK Integration

Extend the `AgentMetadata` in `kamn-sdk` to optionally carry structured tool definitions alongside the existing `capabilities: Vec<String>` for backwards compatibility:

**File:** `crates/kamn-sdk/src/types.rs` — add:
```rust
pub struct AgentToolAdvertisement {
    pub tool_id: String,
    pub display_name: String,
    pub description: String,
}
```

**File:** `crates/kamn-sdk/src/agent.rs` — extend `KamnAgent` trait:
```rust
/// Lists tools available from this agent/node.
fn list_tools(&self) -> Result<Vec<AgentToolAdvertisement>, SdkError>;
```

### 1.5 Acceptance Criteria

- AC-1: `ToolDefinition` validates all fields per contract; invalid inputs produce deterministic errors.
- AC-2: `ToolRegistry` registers/deregisters/lists tools in deterministic BTreeMap order.
- AC-3: Duplicate `tool_id` registration is rejected.
- AC-4: `register_builtin_tools()` registers exactly 13 tools matching the `KamnAgent` trait surface.
- AC-5: Snapshot export/restore round-trips without data loss.
- AC-6: All new code has unit tests, conformance tests, and docs-contract tests.

---

## Phase 2 — MCP Server (stdio transport)

**Goal:** Implement an MCP-compliant JSON-RPC 2.0 server that exposes the tool registry over stdio transport, enabling AI clients (Claude Desktop, Cursor, etc.) to discover and invoke KAMN agent tools.

### 2.1 MCP Protocol Types (`kamn-core`)

**File:** `crates/kamn-core/src/mcp_protocol.rs`

Core JSON-RPC 2.0 types for MCP:

```rust
/// JSON-RPC 2.0 request envelope.
pub struct JsonRpcRequest {
    pub jsonrpc: String,       // Must be "2.0"
    pub id: JsonRpcId,         // number or string
    pub method: String,
    pub params: Option<BTreeMap<String, String>>,
}

/// JSON-RPC 2.0 response envelope.
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: JsonRpcId,
    pub result: Option<String>,   // JSON-encoded result
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 error object.
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<String>,
}

/// MCP server capability declaration.
pub struct McpServerCapabilities {
    pub tools: McpToolCapability,
}

/// MCP tool list response item.
pub struct McpToolDescription {
    pub name: String,
    pub description: String,
    pub input_schema: McpInputSchema,
}

/// MCP input schema (JSON Schema object).
pub struct McpInputSchema {
    pub schema_type: String,    // Always "object"
    pub properties: BTreeMap<String, McpPropertySchema>,
    pub required: Vec<String>,
}

pub struct McpPropertySchema {
    pub schema_type: String,
    pub description: String,
}
```

**Validation:**
- `jsonrpc` must be exactly `"2.0"`.
- `id` must be present for requests (not notifications).
- `method` must be non-empty.
- Parse errors return JSON-RPC error code `-32700`.
- Invalid method returns error code `-32601`.
- Invalid params returns error code `-32602`.

### 2.2 MCP Method Router (`kamn-core`)

**File:** `crates/kamn-core/src/mcp_router.rs`

```rust
/// Routes MCP methods to tool registry and agent operations.
pub struct McpRouter {
    tool_registry: ToolRegistry,
    server_info: McpServerInfo,
}

pub struct McpServerInfo {
    pub name: String,           // "kamn-mcp-server"
    pub version: String,        // Crate version
}

impl McpRouter {
    pub fn new(tool_registry: ToolRegistry, server_info: McpServerInfo) -> Self;

    /// Routes a parsed JSON-RPC request to the appropriate handler.
    /// Returns a JSON-RPC response.
    pub fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse;
}
```

**Supported MCP methods:**

| Method | Handler | Description |
|---|---|---|
| `initialize` | Return server info + capabilities | Handshake |
| `initialized` | No-op acknowledgment | Client ready signal |
| `tools/list` | Query `ToolRegistry::list_tools()` | Tool discovery |
| `tools/call` | Dispatch to tool executor | Tool invocation |
| `ping` | Return `{}` | Health check |

Methods NOT in scope for Phase 2: `resources/*`, `prompts/*`, `sampling/*`, `logging/*`, `completion/*`.

### 2.3 Tool Executor (`kamn-core`)

**File:** `crates/kamn-core/src/mcp_tool_executor.rs`

```rust
/// Executes a tool call by mapping MCP params to KamnAgent trait methods.
pub struct McpToolExecutor<A: KamnAgent> {
    agent: A,
}

impl<A: KamnAgent> McpToolExecutor<A> {
    pub fn new(agent: A) -> Self;

    /// Executes a tool by name with the provided JSON params.
    /// Returns a JSON-encoded result or an MCP-formatted error.
    pub fn execute(
        &mut self,
        tool_name: &str,
        params: &BTreeMap<String, String>,
    ) -> Result<String, McpToolExecutionError>;
}

pub enum McpToolExecutionError {
    UnknownTool { name: String },
    MissingRequiredParam { tool: String, param: String },
    InvalidParamValue { tool: String, param: String, reason: String },
    AgentError { tool: String, error: SdkError },
}
```

The executor maps each `tool_name` to the corresponding `KamnAgent` method, validates/parses the params, calls the method, and serializes the result to JSON.

### 2.4 Stdio Transport (`kamn-node`)

**File:** `crates/kamn-node/src/mcp_stdio.rs`

```rust
/// Runs the MCP server over stdin/stdout using newline-delimited JSON.
pub fn serve_mcp_stdio(router: McpRouter) -> Result<(), String>;
```

**Protocol:**
- Read line-delimited JSON from stdin.
- Parse each line as `JsonRpcRequest`.
- Route through `McpRouter::handle_request()`.
- Write `JsonRpcResponse` as single-line JSON to stdout.
- Log diagnostics to stderr (never stdout — stdout is the MCP wire).

**Runtime mode integration:**
- Add `--runtime-mode mcp-stdio` to `kamn-node` CLI.
- When this mode is selected, the node starts the MCP stdio server instead of the HTTP API.
- The node bootstraps an `InMemoryKamnClient`, registers builtin tools, and serves.

### 2.5 MCP Server Configuration for AI Clients

The MCP server should be invocable via a standard MCP server config block:

```json
{
  "mcpServers": {
    "kamn": {
      "command": "kamn-node",
      "args": ["--runtime-mode", "mcp-stdio", "--role", "processor"],
      "env": {}
    }
  }
}
```

### 2.6 Envelope Translation (`kamn-core`)

**File:** `crates/kamn-core/src/mcp_envelope_translation.rs`

Implements the actual runtime translation between `CanonicalMessageEnvelope` and MCP JSON-RPC messages, moving beyond the static mapping doc:

```rust
/// Translates a KAMN Request envelope into an MCP tools/call request.
pub fn envelope_to_mcp_tool_call(
    envelope: &CanonicalMessageEnvelope,
) -> Result<JsonRpcRequest, McpTranslationError>;

/// Translates an MCP tools/call result into a KAMN Response envelope.
pub fn mcp_tool_result_to_envelope(
    response: &JsonRpcResponse,
    original_envelope: &CanonicalMessageEnvelope,
) -> Result<CanonicalMessageEnvelope, McpTranslationError>;

/// Maps KAMN task states to MCP-oriented status strings.
pub fn task_state_to_mcp_status(state: &str) -> Result<&'static str, McpTranslationError>;

pub enum McpTranslationError {
    UnsupportedMessageType { message_type: String },
    MissingRequiredField { field: String },
    InvalidEnvelopeBody { reason: String },
    AmbiguousMapping { detail: String },
}
```

**Translation rules (codifying the existing mapping doc):**

| KAMN Envelope `message_type` | MCP Method | Direction |
|---|---|---|
| `Request` | `tools/call` | KAMN → MCP |
| `Response` | (tool result) | MCP → KAMN |
| `StatusUpdate` | (notification) | KAMN → MCP |
| All others | `McpTranslationError::UnsupportedMessageType` | Fail-closed |

### 2.7 Acceptance Criteria

- AC-1: `McpRouter` handles `initialize`, `tools/list`, `tools/call`, `ping`, returns valid JSON-RPC 2.0 responses.
- AC-2: `tools/list` returns all 13 builtin tools with correct schemas.
- AC-3: `tools/call` for each builtin tool executes the corresponding `KamnAgent` method and returns the result.
- AC-4: Invalid JSON-RPC requests return appropriate error codes (-32700, -32601, -32602).
- AC-5: Unknown tool names return MCP error with `code: -32602`.
- AC-6: stdio transport reads from stdin, writes to stdout, logs to stderr.
- AC-7: `kamn-node --runtime-mode mcp-stdio` starts the server and handles at least one `initialize` + `tools/list` + `tools/call` sequence.
- AC-8: Envelope translation converts Request envelopes to `tools/call` and Response envelopes from tool results, fail-closed on unsupported types.
- AC-9: Conformance fixture expanded to cover all 13 tool calls + error cases.

---

## Phase 3 — CLI Agent Subcommands

**Goal:** Add a user-facing CLI that wraps `ServiceApiClient` for agent operations, enabling human operators and scripts to interact with a running KAMN node.

### 3.1 CLI Subcommand Structure

Extend `kamn-node` with a subcommand dispatch layer. The existing flag-based CLI remains the default (no subcommand = node operation mode). New subcommands are prefixed for clarity:

```
kamn-node                                          # Existing: start node
kamn-node agent register --type <t> --model <m> --capability <c> [--capability <c>...]
kamn-node agent resolve --did <did>
kamn-node agent search [--capability <c>] [--model <m>]
kamn-node agent reputation --did <did>

kamn-node message send --from <did> --to <did> --body <text> [--channel <id>]
kamn-node message get --id <message-id>

kamn-node task create --creator <did> --type <t> --description <desc>
kamn-node task accept --id <task-id> --assignee <did>
kamn-node task complete --id <task-id>
kamn-node task get --id <task-id>

kamn-node tool list [--provider <did>]
kamn-node tool call --name <tool-id> [--param key=value ...]

kamn-node health
```

### 3.2 Implementation (`kamn-node`)

**File:** `crates/kamn-node/src/cli_subcommands.rs`

```rust
pub enum Subcommand {
    Agent(AgentSubcommand),
    Message(MessageSubcommand),
    Task(TaskSubcommand),
    Tool(ToolSubcommand),
    Health,
}

pub enum AgentSubcommand {
    Register { agent_type: String, model_family: String, capabilities: Vec<String> },
    Resolve { did: String },
    Search { capability: Option<String>, model_family: Option<String> },
    Reputation { did: String },
}

pub enum MessageSubcommand {
    Send { from: String, to: String, body: String, channel: Option<String> },
    Get { message_id: String },
}

pub enum TaskSubcommand {
    Create { creator: String, task_type: String, description: String },
    Accept { task_id: String, assignee: String },
    Complete { task_id: String },
    Get { task_id: String },
}

pub enum ToolSubcommand {
    List { provider: Option<String> },
    Call { name: String, params: Vec<(String, String)> },
}
```

**File:** `crates/kamn-node/src/cli_subcommand_executor.rs`

Each subcommand executor:
1. Parses and validates arguments.
2. Builds a `ServiceRequestAuth` envelope (requires `--auth-did` and `--auth-nonce` flags, or reads from config/env).
3. Calls the corresponding `ServiceApiClient` method.
4. Formats output as text or JSON (respects `--output` flag).

### 3.3 Auth Configuration for CLI

CLI subcommands need DID authentication to call the Service API. Add:

```
--auth-did <did>              # Sender DID for request signing
--auth-nonce <nonce>          # Request nonce (auto-increment if omitted)
--service-endpoint <url>      # Service API base URL (default: http://127.0.0.1:8080)
```

Environment variable overrides: `KAMN_AUTH_DID`, `KAMN_AUTH_NONCE`, `KAMN_SERVICE_ENDPOINT`.

### 3.4 Acceptance Criteria

- AC-1: Each subcommand parses arguments, validates inputs, and calls the correct `ServiceApiClient` method.
- AC-2: `kamn-node agent register` returns a DID on success.
- AC-3: `kamn-node tool list` returns all registered tools in deterministic order.
- AC-4: `kamn-node tool call --name kamn:tool:agent-search:v1 --param capability=analysis` returns matching agents.
- AC-5: Missing required arguments produce clear error messages with usage hints.
- AC-6: `--output json` produces machine-parseable JSON for all subcommands.
- AC-7: `--output text` produces human-readable table/line output.
- AC-8: Auth failure (missing DID, bad signature) surfaces the service API reason code.

---

## Phase 4 — MCP Client and Streaming

**Goal:** Enable KAMN agents to connect to external MCP servers as a client, and add SSE transport for streaming tool results.

### 4.1 MCP Client (`kamn-core`)

**File:** `crates/kamn-core/src/mcp_client.rs`

```rust
/// Client for connecting to external MCP servers.
pub struct McpClient {
    transport: McpClientTransport,
    server_capabilities: Option<McpServerCapabilities>,
}

pub enum McpClientTransport {
    /// Spawns a child process and communicates over stdin/stdout.
    Stdio { command: String, args: Vec<String> },
}

impl McpClient {
    /// Connects to an MCP server, performs initialize handshake.
    pub fn connect(transport: McpClientTransport) -> Result<Self, McpClientError>;

    /// Discovers tools available on the remote server.
    pub fn list_tools(&mut self) -> Result<Vec<McpToolDescription>, McpClientError>;

    /// Invokes a tool on the remote server.
    pub fn call_tool(
        &mut self,
        tool_name: &str,
        params: BTreeMap<String, String>,
    ) -> Result<String, McpClientError>;

    /// Closes the connection gracefully.
    pub fn disconnect(&mut self) -> Result<(), McpClientError>;
}
```

### 4.2 SSE Transport for MCP Server (`kamn-node`)

**File:** `crates/kamn-node/src/mcp_sse.rs`

Add `--runtime-mode mcp-sse` that serves the MCP protocol over HTTP with Server-Sent Events for streaming responses. This enables web-based MCP clients.

**Routes:**
- `POST /mcp` — Accept JSON-RPC requests, return JSON-RPC responses.
- `GET /mcp/sse` — SSE endpoint for streaming tool results and notifications.

### 4.3 External Tool Bridge

**File:** `crates/kamn-core/src/mcp_external_tool_bridge.rs`

Register tools discovered from external MCP servers into the local `ToolRegistry` with a `provider_did` of `kamn:did:external:<server-name>`. This allows `tools/list` to aggregate both local and remote tools.

### 4.4 Acceptance Criteria

- AC-1: `McpClient` can spawn a child process, perform `initialize` handshake, call `tools/list`, and invoke `tools/call`.
- AC-2: `McpClient` handles JSON-RPC errors from remote servers without panicking.
- AC-3: SSE transport serves MCP over HTTP, returns streaming tool results.
- AC-4: External tools appear in `tools/list` with `external` provider prefix.
- AC-5: Disconnect gracefully terminates child process / SSE connection.

---

## Module Placement Summary

| Module | Crate | File Path | Phase |
|---|---|---|---|
| `tool_definition` | `kamn-core` | `crates/kamn-core/src/tool_definition.rs` | 1 |
| `tool_registry` | `kamn-core` | `crates/kamn-core/src/tool_registry.rs` | 1 |
| `tool_builtins` | `kamn-core` | `crates/kamn-core/src/tool_builtins.rs` | 1 |
| `mcp_protocol` | `kamn-core` | `crates/kamn-core/src/mcp_protocol.rs` | 2 |
| `mcp_router` | `kamn-core` | `crates/kamn-core/src/mcp_router.rs` | 2 |
| `mcp_tool_executor` | `kamn-core` | `crates/kamn-core/src/mcp_tool_executor.rs` | 2 |
| `mcp_envelope_translation` | `kamn-core` | `crates/kamn-core/src/mcp_envelope_translation.rs` | 2 |
| `mcp_stdio` | `kamn-node` | `crates/kamn-node/src/mcp_stdio.rs` | 2 |
| `cli_subcommands` | `kamn-node` | `crates/kamn-node/src/cli_subcommands.rs` | 3 |
| `cli_subcommand_executor` | `kamn-node` | `crates/kamn-node/src/cli_subcommand_executor.rs` | 3 |
| `mcp_client` | `kamn-core` | `crates/kamn-core/src/mcp_client.rs` | 4 |
| `mcp_sse` | `kamn-node` | `crates/kamn-node/src/mcp_sse.rs` | 4 |
| `mcp_external_tool_bridge` | `kamn-core` | `crates/kamn-core/src/mcp_external_tool_bridge.rs` | 4 |

## Test Artifacts per Phase

| Phase | Test Files |
|---|---|
| 1 | `crates/kamn-core/tests/tool_definition.rs`, `crates/kamn-core/tests/tool_registry.rs`, `crates/kamn-core/tests/tool_builtins.rs` |
| 2 | `crates/kamn-core/tests/mcp_protocol.rs`, `crates/kamn-core/tests/mcp_router.rs`, `crates/kamn-core/tests/mcp_tool_executor.rs`, `crates/kamn-core/tests/mcp_envelope_translation.rs`, `crates/kamn-node/tests/mcp_stdio_integration.rs` |
| 3 | `crates/kamn-node/tests/cli_subcommand_tests.rs`, `crates/kamn-node/tests/cli_subcommand_executor_tests.rs` |
| 4 | `crates/kamn-core/tests/mcp_client.rs`, `crates/kamn-node/tests/mcp_sse_integration.rs` |

## Documentation Artifacts per Phase

| Phase | Doc Files |
|---|---|
| 1 | `docs/foundation/tool-registry.md` |
| 2 | `docs/foundation/mcp-server.md`, update `docs/foundation/a2a-mcp-interoperability.md` |
| 3 | `docs/foundation/cli-agent-subcommands.md`, update `docs/foundation/node-runtime-cli.md` |
| 4 | `docs/foundation/mcp-client.md` |

## Conformance Harness Updates

| Phase | Fixture/Script Changes |
|---|---|
| 1 | New fixture: `fixtures/tool_registry/tool_definition_cases.json` |
| 2 | Expand `fixtures/a2a_mcp_conformance/replay_cases.json` from 4 → 20+ cases covering all 13 tools + error paths. New: `scripts/mcp/run_mcp_protocol_conformance_harness.py` |
| 3 | New fixture: `fixtures/cli_subcommands/subcommand_cases.json` |
| 4 | New fixture: `fixtures/mcp_client/client_integration_cases.json` |

---

## Dependency and Risk Assessment

### Dependencies

- **Phase 1** has zero external dependencies. Pure `kamn-core` types and `BTreeMap` storage.
- **Phase 2** stdio transport requires only `std::io` — no new crate dependencies.
- **Phase 2** JSON serialization: evaluate whether to add `serde_json` (currently the codebase uses hand-rolled JSON parsing in `service.rs`). Recommendation: use `serde_json` for MCP protocol types since JSON-RPC 2.0 requires precise JSON handling. This is a new dependency requiring approval per AGENTS.md §2.
- **Phase 3** builds on existing `ServiceApiClient` — no new external dependencies.
- **Phase 4** child process spawning uses `std::process::Command` — no new dependencies. SSE transport can reuse the existing `axum` server infrastructure.

### Risks

| Risk | Mitigation |
|---|---|
| `serde_json` dependency approval | Hand-rolled JSON is already fragile in `service.rs`; `serde_json` is a standard Rust ecosystem crate with no transitive risk. Prepare ADR. |
| MCP spec drift | Pin to MCP specification version 2024-11-05. Add schema version marker to `McpServerInfo`. |
| Tool executor becomes a God object | Keep executor as a thin dispatch layer — each tool maps 1:1 to a `KamnAgent` method. No business logic in the executor. |
| CLI subcommand parsing complexity | Reuse the existing hand-rolled parser pattern from `cli.rs`. Consider `clap` only if complexity grows beyond 3 subcommand levels. |
| Backwards compatibility for `AgentMetadata.capabilities` | Keep `Vec<String>` as-is. Add `tools: Vec<AgentToolAdvertisement>` alongside. Existing code that reads `capabilities` is unaffected. |

---

## Effort Estimates

| Phase | New Rust LOC (est.) | New Test LOC (est.) | New Doc LOC (est.) |
|---|---|---|---|
| 1 — Tool Definitions + Registry | 400–600 | 300–500 | 100–150 |
| 2 — MCP Server (stdio) | 800–1200 | 600–900 | 200–300 |
| 3 — CLI Subcommands | 500–800 | 400–600 | 150–200 |
| 4 — MCP Client + SSE | 600–900 | 400–600 | 150–200 |
| **Total** | **2300–3500** | **1700–2600** | **600–850** |

## Phase Ordering and Parallelism

```
Phase 1 (Tool Definitions + Registry)
    │
    ├──→ Phase 2 (MCP Server)     ← depends on Phase 1
    │
    └──→ Phase 3 (CLI Subcommands) ← independent of Phase 2
              │
              └──→ Phase 4 (MCP Client + SSE) ← depends on Phase 2
```

Phases 2 and 3 can be developed in parallel after Phase 1 is complete.
