# PRD: End-to-End Live Testing with Portable Agent Integration

**Document version:** 3.0
**Date:** 2026-02-21
**Status:** Draft
**Author:** Review R50 PRD generation
**Revision history:**
- v1 — OpenClaw-based agent integration
- v2 — Tau-first with tau-kamn-bridge translation layer
- v3 — Portable architecture: KAMN-owned MCP server + CLI, any agent can consume

---

## 1. Problem Statement

KAMN has 3,379 unit/integration tests, 92 production modules, and comprehensive contract test suites — but **zero live end-to-end validation with real agents performing real operations across a real network**. All existing tests use in-memory transports, mock clients, and deterministic fixtures. The codebase has never been validated as a working system where:

- Real AI agents discover each other, negotiate, exchange encrypted messages, complete tasks, and settle payments
- Proof anchoring persists to a real Kolme blockchain and can be independently verified
- The full message lifecycle (Created → Signed → Encrypted → Broadcast → Delivered → Validated → Anchored) executes across real network boundaries
- System behavior under failure (node crashes, network partitions, signer unavailability) is observed with real processes

Additionally, KAMN has no standard interface for AI agents to interact with it. The `KamnAgent` trait and Service API exist but require agent-side code to handle authentication headers, envelope construction, and encryption — protocol details that should be KAMN's responsibility, not the agent's.

This PRD defines:
1. A **portable agent integration layer** — an MCP server and CLI that any AI agent can use to interact with KAMN
2. A **live end-to-end test system** that validates every layer of KAMN with verifiable, reproducible evidence

---

## 2. Goals

1. **Any AI agent can use KAMN** — ship an MCP server and CLI so any MCP-compatible agent (Claude Code, Tau, Cursor, custom agents) gets full KAMN capability with zero agent-side code
2. **KAMN owns its interface** — authentication, envelope construction, encryption, and protocol details are handled by KAMN's own libraries, not reimplemented per-agent
3. **Validate the complete KAMN stack** end-to-end with real processes, real network I/O, real storage, and real blockchain anchoring
4. **Produce verifiable proof** — every test run generates an evidence bundle with Kolme block hashes, message proof anchors, signed receipts, and deterministic replay markers
5. **Be repeatable** — the entire system can be torn down and rebuilt from scratch, producing identical functional outcomes
6. **Cover every major subsystem** — identity, messaging, channels, tasks, escrow, transport, anchoring, storage, recovery

---

## 3. Non-Goals

- Production deployment readiness (this is a test/demo environment)
- Performance benchmarking at scale (covered by existing capacity load tests)
- Security penetration testing
- UI/frontend validation
- Replacing the KAMN Service API (the MCP server and CLI are clients of it)

---

## 4. Portable Agent Integration Architecture

### 4.1 Design Principle

KAMN protocol complexity (DID formats, signature schemes, envelope construction, encryption) should be encapsulated by KAMN, not pushed onto every agent integration. An agent should be able to say "send a message to Bob" without knowing about `x-kamn-request-nonce` headers or `XChaCha20-Poly1305` ciphertexts.

### 4.2 Three-Layer Architecture

```
┌───────────────────────────────────────────────────────────────────────┐
│                        AI Agent Runtimes                              │
│                                                                       │
│  Claude Code    Tau    Cursor    Windsurf    Custom Agents    Scripts  │
│       │          │       │          │             │              │     │
│       └──────────┴───────┴──────────┴─────────────┴──────────────┘     │
│                          │                        │                    │
│                     MCP protocol              shell exec               │
│                    (stdio / SSE)             (JSON output)              │
└──────────────────────────┬────────────────────────┬────────────────────┘
                           │                        │
               ┌───────────▼──────────┐  ┌─────────▼──────────┐
               │  kamn-mcp-server     │  │  kamn-cli           │
               │  (MCP tool server)   │  │  (command-line)     │
               │                      │  │                     │
               │  12 MCP tools:       │  │  12 subcommands:    │
               │    kamn_register     │  │    register          │
               │    kamn_send_message │  │    send-message      │
               │    kamn_create_chan  │  │    create-channel    │
               │    kamn_create_task  │  │    create-task       │
               │    kamn_accept_task  │  │    accept-task       │
               │    kamn_complete_task│  │    complete-task      │
               │    kamn_fund_escrow  │  │    fund-escrow       │
               │    kamn_release_esc  │  │    release-escrow    │
               │    kamn_query_msg    │  │    query-message     │
               │    kamn_list_msgs    │  │    list-messages     │
               │    kamn_verify_proof │  │    verify-proof      │
               │    kamn_health       │  │    health             │
               └───────────┬──────────┘  └─────────┬──────────┘
                           │                        │
                           └────────────┬───────────┘
                                        │
                              ┌─────────▼──────────┐
                              │  kamn-agent-lib     │
                              │  (shared library)   │
                              │                     │
                              │  - DID management   │
                              │  - Auth headers     │
                              │  - Envelope builder │
                              │  - Encryption       │
                              │  - Service API      │
                              │    HTTP client      │
                              │  - Kolme proof      │
                              │    verification     │
                              │  - Key management   │
                              │  - Nonce tracking   │
                              └─────────┬──────────┘
                                        │
                                        │ HTTP + x-kamn-* headers
                                        ▼
                              ┌───────────────────────┐
                              │  KAMN Service API      │
                              │  (Axum, port 8080+)    │
                              │  (existing, unchanged) │
                              └───────────────────────┘
```

### 4.3 `kamn-agent-lib` — Shared Core Library

A new library crate in the KAMN repo that encapsulates all protocol details needed to interact with KAMN as an agent. Both the MCP server and CLI are thin wrappers over this library.

```rust
// crates/kamn-agent-lib/src/lib.rs

pub mod identity;      // DID generation, key management, key derivation
pub mod auth;          // KAMN auth header construction + signature generation
pub mod envelope;      // CanonicalMessageEnvelope builder + encryption
pub mod client;        // Typed HTTP client for all Service API routes
pub mod kolme;         // Kolme proof fetching + independent verification
pub mod nonce;         // Monotonic nonce tracking per agent
pub mod errors;        // Error taxonomy

/// Top-level agent handle. Manages identity, nonce state, and API client.
pub struct KamnAgentHandle {
    identity: AgentIdentity,
    client: ServiceApiHttpClient,
    nonce_counter: AtomicU64,
    kolme_client: KolmeClient,
}
```

**Identity management:**

```rust
// crates/kamn-agent-lib/src/identity.rs

/// Agent identity with ed25519 signing key and derived X25519 encryption key.
pub struct AgentIdentity {
    pub kamn_did: String,           // kamn:did:agent:{name}
    pub signing_key: ed25519_dalek::SigningKey,
    pub x25519_secret: x25519_dalek::StaticSecret,
}

impl AgentIdentity {
    /// Create a new agent identity with a generated key pair.
    pub fn generate(name: &str) -> Self { /* ... */ }

    /// Load identity from a key file (PKCS8 or raw bytes).
    pub fn from_key_file(name: &str, path: &Path) -> Result<Self, IdentityError> { /* ... */ }

    /// Load identity from environment variables.
    /// Reads KAMN_AGENT_DID and KAMN_AGENT_PRIVATE_KEY_HEX.
    pub fn from_env() -> Result<Self, IdentityError> { /* ... */ }
}
```

**Auth header construction:**

```rust
// crates/kamn-agent-lib/src/auth.rs

/// KAMN signature format:
///   sig:ed25519:baseline-v1:{sender_did}:{nonce}:{state_hash}:{body_len}
pub struct KamnAuthHeaders {
    pub sender_did: String,
    pub request_nonce: String,
    pub request_signature: String,
}

impl KamnAuthHeaders {
    pub fn build(
        signing_key: &ed25519_dalek::SigningKey,
        kamn_did: &str,
        nonce: u64,
        state_hash: &str,
        body: &[u8],
    ) -> Self { /* ... */ }

    pub fn apply(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
            .header("x-kamn-sender-did", &self.sender_did)
            .header("x-kamn-request-nonce", &self.request_nonce)
            .header("x-kamn-request-signature", &self.request_signature)
    }
}
```

**Envelope construction + encryption:**

```rust
// crates/kamn-agent-lib/src/envelope.rs

/// Build a CanonicalMessageEnvelope from plaintext.
///
/// Handles: payload hashing, X25519 key agreement, XChaCha20-Poly1305
/// encryption, canonical JSON serialization, and ed25519 signing.
pub fn build_and_sign_envelope(
    sender: &AgentIdentity,
    nonce: u64,
    recipient_did: &str,
    recipient_pubkey: &x25519_dalek::PublicKey,
    channel_id: &str,
    plaintext: &[u8],
) -> Result<SignedEnvelope, EnvelopeError> { /* ... */ }
```

**Service API HTTP client:**

```rust
// crates/kamn-agent-lib/src/client.rs

pub struct ServiceApiHttpClient {
    base_url: String,
    http: reqwest::Client,
}

impl ServiceApiHttpClient {
    // All routes from KAMN Service API:
    pub async fn bootstrap_agent(&self, agent: &AgentIdentity, metadata: &AgentMetadata) -> Result<BootstrapReceipt, ClientError>;
    pub async fn send_message(&self, agent: &AgentIdentity, nonce: u64, to: &str, payload: &str, channel_id: Option<&str>) -> Result<SendReceipt, ClientError>;
    pub async fn get_message(&self, agent: &AgentIdentity, nonce: u64, message_id: &str) -> Result<MessageStatus, ClientError>;
    pub async fn create_channel(&self, agent: &AgentIdentity, nonce: u64, channel_type: ChannelType, members: &[String]) -> Result<ChannelReceipt, ClientError>;
    pub async fn get_channel(&self, agent: &AgentIdentity, nonce: u64, channel_id: &str) -> Result<ChannelInfo, ClientError>;
    pub async fn list_messages(&self, agent: &AgentIdentity, nonce: u64, channel_id: &str) -> Result<Vec<MessageSummary>, ClientError>;
    pub async fn create_task(&self, agent: &AgentIdentity, nonce: u64, task: &TaskDefinition) -> Result<TaskReceipt, ClientError>;
    pub async fn get_task(&self, agent: &AgentIdentity, nonce: u64, task_id: &str) -> Result<TaskStatus, ClientError>;
    pub async fn accept_task(&self, agent: &AgentIdentity, nonce: u64, task_id: &str) -> Result<TaskReceipt, ClientError>;
    pub async fn complete_task(&self, agent: &AgentIdentity, nonce: u64, task_id: &str, evidence: &str) -> Result<TaskReceipt, ClientError>;
    pub async fn fund_escrow(&self, agent: &AgentIdentity, nonce: u64, task_id: &str, amount: u64) -> Result<EscrowReceipt, ClientError>;
    pub async fn release_escrow(&self, agent: &AgentIdentity, nonce: u64, escrow_id: &str) -> Result<EscrowReceipt, ClientError>;
    pub async fn health(&self) -> Result<HealthStatus, ClientError>;
}
```

### 4.4 `kamn-mcp-server` — MCP Tool Server

An MCP server binary that exposes all KAMN operations as MCP tools. Any MCP-compatible agent connects to it and gets full KAMN capability.

**Configuration:**

```json
{
  "mcpServers": {
    "kamn": {
      "command": "kamn-mcp-server",
      "args": [
        "--endpoint", "http://localhost:8080",
        "--agent-name", "alice",
        "--key-file", "/path/to/alice.key"
      ]
    }
  }
}
```

Or with environment-based identity:

```json
{
  "mcpServers": {
    "kamn": {
      "command": "kamn-mcp-server",
      "args": ["--endpoint", "http://localhost:8080"],
      "env": {
        "KAMN_AGENT_DID": "kamn:did:agent:alice-e2e-1",
        "KAMN_AGENT_PRIVATE_KEY_HEX": "..."
      }
    }
  }
}
```

**Implementation Progress (2026-02-22):**

| Tool | Status | Notes |
|---|---|---|
| `register` | Implemented | Returns current agent DID via `kamn-agent-lib` identity. |
| `send_message` | Implemented | Dispatches through `KamnAgentHandle::send_message`. |
| `create_channel` | Implemented | Dispatches through `KamnAgentHandle::create_channel`. |
| `list_messages` | Implemented | Backed by `GET /v1/channels/{id}/messages` via `kamn-sdk` + `kamn-agent-lib`. |
| `query_message` | Implemented | Dispatches through `KamnAgentHandle::query_message`. |
| `create_task` | Implemented | Dispatches through `KamnAgentHandle::create_task`. |
| `health` | Implemented | Dispatches through `KamnAgentHandle::health`. |
| `accept_task` | Implemented | Dispatches through `KamnAgentHandle::accept_task` with deterministic task-id argument validation. |
| `complete_task` | Implemented | Dispatches through `KamnAgentHandle::complete_task` with deterministic task-id argument validation. |
| `fund_escrow` | Implemented | Dispatches through `KamnAgentHandle::fund_escrow` with deterministic payload argument validation. |
| `release_escrow` | Implemented | Dispatches through `KamnAgentHandle::release_escrow` with deterministic escrow-id argument validation. |
| `verify_proof` | Implemented | Dispatches through `KamnAgentHandle::verify_proof` with deterministic invalid-request handling for malformed payload fields. |

Current `kamn-cli` activation status for the same supported surface:
- Implemented: `register`, `send-message`, `create-channel`, `list-messages`, `query-message`, `create-task`, `verify-proof`, `health`
- Implemented: `register`, `send-message`, `create-channel`, `list-messages`, `query-message`, `create-task`, `accept-task`, `complete-task`, `fund-escrow`, `release-escrow`, `verify-proof`, `health`

**MCP Tool Definitions:**

Each KAMN operation becomes an MCP tool with a JSON Schema parameter definition:

```rust
// crates/kamn-mcp-server/src/tools.rs

fn register_tools(server: &mut McpServer, agent_handle: Arc<KamnAgentHandle>) {
    server.register_tool(McpTool {
        name: "kamn_send_message",
        description: "Send an encrypted message to another KAMN agent. \
            The message is automatically signed, encrypted, and delivered \
            via the KAMN network with Kolme proof anchoring.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Recipient agent DID (e.g. kamn:did:agent:bob-1)"
                },
                "message": {
                    "type": "string",
                    "description": "Message content to send"
                },
                "channel_id": {
                    "type": "string",
                    "description": "Channel ID. Omit for auto-created direct channel."
                }
            },
            "required": ["to", "message"]
        }),
        handler: move |params| {
            let handle = agent_handle.clone();
            async move {
                let to = params["to"].as_str().unwrap();
                let message = params["message"].as_str().unwrap();
                let channel_id = params.get("channel_id").and_then(|v| v.as_str());
                let result = handle.send_message(to, message, channel_id).await?;
                Ok(json!({
                    "message_id": result.message_id,
                    "status": result.lifecycle_status,
                    "channel_id": result.channel_id,
                    "timestamp": result.timestamp
                }))
            }
        }
    });

    server.register_tool(McpTool {
        name: "kamn_register",
        description: "Register this agent's DID on the KAMN network. \
            Must be called once before other operations.",
        input_schema: json!({
            "type": "object",
            "properties": {
                "display_name": {
                    "type": "string",
                    "description": "Human-readable agent name"
                }
            }
        }),
        handler: /* ... */
    });

    // ... remaining 10 tools follow same pattern
}
```

**Full MCP tool list:**

| MCP Tool | Description | Required Params | Optional Params |
|----------|-------------|-----------------|-----------------|
| `kamn_register` | Register agent DID on the network | — | `display_name` |
| `kamn_send_message` | Send encrypted message | `to`, `message` | `channel_id` |
| `kamn_create_channel` | Create direct or group channel | `channel_type`, `members` | `name` |
| `kamn_list_messages` | List messages in a channel | `channel_id` | `limit`, `offset` |
| `kamn_query_message` | Get message status + proof status | `message_id` | — |
| `kamn_create_task` | Create a task with optional escrow | `description`, `criteria` | `escrow_amount`, `deadline` |
| `kamn_accept_task` | Accept a pending task | `task_id` | — |
| `kamn_complete_task` | Submit completion evidence | `task_id`, `evidence` | — |
| `kamn_fund_escrow` | Fund escrow for a task | `task_id`, `amount` | — |
| `kamn_release_escrow` | Release escrowed funds | `escrow_id` | — |
| `kamn_verify_proof` | Verify Kolme blockchain anchor | `message_id` | `expected_block_height` |
| `kamn_health` | Check KAMN network health | — | — |

### 4.5 `kamn-cli` — Command-Line Interface

A CLI binary for human operators, shell scripts, and agents that prefer shell execution over MCP:

```bash
# Register an agent
kamn-cli register \
  --endpoint http://localhost:8080 \
  --agent-name alice \
  --key-file alice.key \
  --display-name "Alice (E2E)"

# Send a message
kamn-cli send-message \
  --endpoint http://localhost:8080 \
  --agent-name alice \
  --key-file alice.key \
  --to kamn:did:agent:bob-1 \
  --message "Hello Bob, E2E test run abc123"

# Query message status
kamn-cli query-message \
  --endpoint http://localhost:8080 \
  --agent-name alice \
  --key-file alice.key \
  --message-id msg-abc123

# Verify Kolme proof
kamn-cli verify-proof \
  --endpoint http://localhost:8080 \
  --kolme-endpoint http://localhost:3000 \
  --message-id msg-abc123
```

**Output format:** All commands output JSON by default, with `--format text` for human-readable output:

```json
{
  "message_id": "msg-abc123",
  "status": "Delivered",
  "channel_id": "chan-xyz789",
  "proof": {
    "finality": "FINAL",
    "kolme_block_height": 42,
    "kolme_block_hash": "sha256:..."
  },
  "timestamp": "2026-02-21T14:31:02Z"
}
```

**Environment variable support:** All connection parameters can be set via env vars to avoid repetition:

```bash
export KAMN_ENDPOINT=http://localhost:8080
export KAMN_KOLME_ENDPOINT=http://localhost:3000
export KAMN_AGENT_DID=kamn:did:agent:alice-e2e-1
export KAMN_AGENT_PRIVATE_KEY_HEX=...

# Now commands are concise:
kamn-cli send-message --to kamn:did:agent:bob-1 --message "Hello"
kamn-cli verify-proof --message-id msg-abc123
```

### 4.6 Agent Skill / Prompt Definition

A portable skill definition that teaches any LLM agent how to use the KAMN MCP tools effectively. This lives in the KAMN repo and can be loaded into any agent runtime:

```markdown
# KAMN Agent Skill

You have access to KAMN network tools for secure agent-to-agent communication.

## Available Tools

- `kamn_register` — Register your identity on the KAMN network (call once at start)
- `kamn_send_message` — Send an encrypted message to another agent
- `kamn_create_channel` — Create a direct or group communication channel
- `kamn_list_messages` — Read messages in a channel
- `kamn_query_message` — Check message delivery and proof status
- `kamn_create_task` — Post a task for other agents to complete
- `kamn_accept_task` — Accept a pending task
- `kamn_complete_task` — Submit task completion evidence
- `kamn_fund_escrow` — Lock funds in escrow for a task
- `kamn_release_escrow` — Release escrowed funds to the performer
- `kamn_verify_proof` — Verify a message or transaction is anchored on the Kolme blockchain
- `kamn_health` — Check network status

## Typical Workflows

### Send a message:
1. `kamn_register` (if not already registered)
2. `kamn_create_channel` with the recipient
3. `kamn_send_message` with the channel_id
4. `kamn_query_message` to confirm delivery
5. `kamn_verify_proof` to confirm blockchain anchoring

### Complete a task:
1. `kamn_register` (if not already registered)
2. `kamn_create_task` with description, criteria, and escrow amount
3. (Performer) `kamn_accept_task`
4. (Performer) `kamn_complete_task` with evidence
5. (Requester) `kamn_release_escrow`
6. `kamn_verify_proof` on the settlement

## Important Notes

- All messages are automatically encrypted end-to-end
- All operations are signed with your agent's private key
- Critical operations are anchored on the Kolme blockchain for verifiability
- Use `kamn_verify_proof` to independently verify any anchored operation
```

This skill file can be:
- Loaded into Claude Code as an MCP server description
- Loaded into Tau as an agent system prompt
- Used as documentation for any custom agent integration
- Included in OpenClaw or other agent frameworks as a plugin guide

---

## 5. System Architecture for E2E Testing

### 5.1 Component Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                     E2E Test Orchestrator                           │
│  (Rust binary: kamn-e2e-harness)                                   │
│  - Starts/stops all infrastructure                                  │
│  - Spawns kamn-mcp-server per agent (or uses kamn-cli)              │
│  - Drives scenarios via agent sessions                              │
│  - Collects evidence bundles                                        │
│  - Produces verification reports                                    │
└──────────────────────────┬──────────────────────────────────────────┘
                           │ orchestrates
           ┌───────────────┼───────────────┐
           │               │               │
           ▼               ▼               ▼
┌──────────────────┐ ┌──────────────┐ ┌──────────────────────────┐
│  KAMN Network    │ │  Kolme       │ │  Agent Fleet              │
│  (3+ nodes)      │ │  Devnet      │ │                          │
│                  │ │              │ │  ┌────────────────────┐   │
│  Processor node  │ │  Processor   │ │  │ Agent "Alice"      │   │
│  Listener node   │ │  API server  │ │  │ (Tau / Claude /    │   │
│  Approver node   │ │  (port 3000) │ │  │  any MCP client)   │   │
│                  │ │              │ │  │        │            │   │
│  Service API     │ │  PostgreSQL  │ │  │   MCP protocol      │   │
│  (port 8080+)    │ │  (storage)   │ │  │        │            │   │
│                  │ │              │ │  │  kamn-mcp-server    │   │
│  libp2p gossip   │ │  WebSocket   │ │  │  (per-agent)       │   │
│  (port 9000+)    │ │  notifs      │ │  └────────┬───────────┘   │
│                  │ │              │ │           │               │
│  PostgreSQL      │ │              │ │  (same for Bob, Carol)   │
│  SQLite          │ │              │ │                          │
└──────────────────┘ └──────────────┘ └──────────┬───────────────┘
         │                    │                   │
         └────────────────────┴───────────────────┘
                              │
                    ┌─────────▼──────────┐
                    │  Evidence Store    │
                    │  (local directory) │
                    │                   │
                    │  - Kolme blocks   │
                    │  - Proof anchors  │
                    │  - Message traces │
                    │  - Agent logs     │
                    │  - Signed receipts│
                    │  - Replay markers │
                    └───────────────────┘
```

### 5.2 Per-Agent MCP Server Instances

Each E2E test agent gets its own `kamn-mcp-server` process with its own identity:

```
Orchestrator spawns:

  kamn-mcp-server --endpoint http://localhost:8080 \
                  --agent-name alice --key-file /tmp/e2e/alice.key
                  → stdio MCP server for Alice's agent runtime

  kamn-mcp-server --endpoint http://localhost:8080 \
                  --agent-name bob --key-file /tmp/e2e/bob.key
                  → stdio MCP server for Bob's agent runtime

  kamn-mcp-server --endpoint http://localhost:8080 \
                  --agent-name carol --key-file /tmp/e2e/carol.key
                  → stdio MCP server for Carol's agent runtime
```

The agent runtime (Tau, Claude Code, etc.) connects to its dedicated MCP server. The server handles all protocol complexity — the agent just calls tools with simple parameters.

### 5.3 Infrastructure Layer

| Component | Implementation | Purpose |
|-----------|---------------|---------|
| Kolme Devnet | Local Kolme node (in-memory or Fjall store) | Blockchain for proof anchoring |
| PostgreSQL | Docker container (port 5432) | KAMN + Kolme persistent storage |
| KAMN Processor | `kamn-node --role processor --runtime-mode full` | Block production, message validation |
| KAMN Listener | `kamn-node --role listener --runtime-mode full` | External event monitoring |
| KAMN Approver | `kamn-node --role approver --runtime-mode full` | Outgoing action authorization |
| kamn-mcp-server (x3) | One per agent, stdio MCP protocol | Agent ↔ KAMN translation |
| Agent Runtime | Tau, Claude Code, or any MCP client | AI agent execution |
| Evidence Store | Local filesystem | Artifact collection |

---

## 6. Agent-Agnostic Execution

### 6.1 Execution Modes

The harness supports four execution modes, all producing identical evidence bundles:

| Mode | Driver | Agent Runtime | Speed | Fidelity | Use Case |
|------|--------|--------------|-------|----------|----------|
| `sdk-direct` | Rust code via `kamn-agent-lib` | None | Fast (< 2 min) | Deterministic | CI weekly gate |
| `cli-scripted` | Shell scripts via `kamn-cli` | None | Fast (< 2 min) | Deterministic | Portable CI, debugging |
| `mcp-tau` | Tau agents via MCP | Tau | Medium (5-15 min) | Non-deterministic | Nightly deep validation |
| `mcp-any` | Any MCP-compatible agent | Configurable | Variable | Non-deterministic | Ad-hoc agent testing |

### 6.2 SDK-Direct Mode (No Agent)

Calls `kamn-agent-lib` directly from Rust — no MCP server, no agent runtime, no LLM:

```rust
let alice = KamnAgentHandle::new(
    AgentIdentity::generate("alice-e2e"),
    "http://localhost:8080",
);
alice.register(None).await?;

let channel = alice.create_channel(ChannelType::Direct, &[bob_did]).await?;
let msg = alice.send_message(&bob_did, "Hello Bob", Some(&channel.id)).await?;
assert_eq!(msg.status, "Delivered");

let proof = alice.verify_proof(&msg.message_id).await?;
assert_eq!(proof.finality, "FINAL");
```

### 6.3 CLI-Scripted Mode

Shell scripts call `kamn-cli` commands. Useful for portable CI and for debugging:

```bash
#!/bin/bash
set -euo pipefail

ALICE_KEY=/tmp/e2e/alice.key
BOB_DID="kamn:did:agent:bob-e2e-1"

# Register
kamn-cli register --key-file "$ALICE_KEY" --display-name "Alice E2E"

# Create channel + send message
CHANNEL=$(kamn-cli create-channel --key-file "$ALICE_KEY" \
  --type direct --members "$BOB_DID" | jq -r .channel_id)

RESULT=$(kamn-cli send-message --key-file "$ALICE_KEY" \
  --to "$BOB_DID" --channel-id "$CHANNEL" \
  --message "Hello Bob, run $RUN_ID")

MSG_ID=$(echo "$RESULT" | jq -r .message_id)

# Verify
kamn-cli verify-proof --message-id "$MSG_ID" | jq -e '.finality == "FINAL"'
```

### 6.4 MCP-Tau Mode

Tau agents connect to per-agent `kamn-mcp-server` instances:

```
┌──────────────────┐     MCP stdio     ┌─────────────────┐     HTTP     ┌─────────────┐
│  Tau Agent       │ ◄────────────────► │ kamn-mcp-server │ ───────────► │ KAMN Service │
│  "Alice"         │                    │ (alice identity)│              │ API          │
│                  │                    │                 │              │              │
│  System prompt:  │                    │ Handles:        │              │ Validates:   │
│  "You are Alice, │                    │ - Auth headers  │              │ - Signature  │
│   use kamn_*     │                    │ - Encryption    │              │ - Nonce      │
│   tools to..."   │                    │ - Envelopes     │              │ - Scope      │
└──────────────────┘                    └─────────────────┘              └─────────────┘
```

The Tau agent only needs to know about the MCP tools — it calls `kamn_send_message` with `{to, message}` and gets back `{message_id, status}`. No KAMN protocol knowledge required.

### 6.5 MCP-Any Mode

Any MCP-compatible agent can be used. The harness spawns `kamn-mcp-server` instances and the user configures their preferred agent runtime to connect:

```json
{
  "mcpServers": {
    "kamn-alice": {
      "command": "kamn-mcp-server",
      "args": ["--endpoint", "http://localhost:8080",
               "--agent-name", "alice",
               "--key-file", "/tmp/e2e/alice.key"]
    }
  }
}
```

This mode is not orchestrated by the harness — it is for interactive testing and demos where a human drives an agent that uses KAMN.

---

## 7. Test Scenarios

### 7.1 Scenario Matrix

| ID | Scenario | Subsystems Validated | Agents | Priority |
|----|----------|---------------------|--------|----------|
| S-01 | Agent Discovery & Identity | DID registry, bootstrap, agent registration | Alice, Bob | P0 |
| S-02 | Direct Message Round-Trip | Encryption, signing, delivery, proof anchoring | Alice, Bob | P0 |
| S-03 | Group Channel Messaging | Channel creation, membership, group crypto | Alice, Bob, Carol | P0 |
| S-04 | Task Lifecycle (Full) | Task creation, acceptance, completion, escrow | Alice, Bob | P0 |
| S-05 | Escrow Settlement | Escrow funding, release, dispute, resolution | Alice, Bob, Carol | P0 |
| S-06 | Kolme Proof Verification | Proof anchoring, block verification, finality | Alice | P0 |
| S-07 | Message Replay Protection | Nonce guards, acceptance window, duplicate rejection | Alice, Bob | P1 |
| S-08 | Node Crash Recovery | Snapshot persistence, state recovery, re-sync, SQLite WAL | Alice, Bob | P1 |
| S-09 | Transport Failover | libp2p reconnect, fallback transport | Alice, Bob | P1 |
| S-10 | Multi-Node Topology Coherence | Cross-node state agreement, gossip propagation | Alice, Bob, Carol | P1 |
| S-11 | Signer Key Rotation | Key lifecycle, rotation handshake, continuity | Alice | P2 |
| S-12 | Content Retention & Deletion | Retention policies, tombstone lifecycle, redaction | Alice, Bob | P2 |
| S-13 | Bridge Message Forwarding | Cross-chain bridge ingress/egress | Alice | P2 |
| S-14 | Batch Merkle Anchoring | M1 batch scheduler, Merkle root anchoring | Alice, Bob | P2 |
| S-15 | Performance Smoke | Throughput budget, latency P50/P99, error rate | Alice, Bob | P2 |

### 7.2 S-01: Agent Discovery & Identity

**Purpose:** Validate that agents can register DIDs and discover each other via the KAMN network.

**Steps (expressed as MCP tool calls):**
1. Alice: `kamn_register` with display_name "Alice E2E"
2. Bob: `kamn_register` with display_name "Bob E2E"
3. Alice: `kamn_health` to verify network connectivity
4. Alice: `kamn_create_channel` (direct) with Bob — implicitly discovers Bob's DID
5. Bob: `kamn_list_messages` on the channel — implicitly confirms discovery

**Verifiable Outputs:**
- `evidence/s01/alice_registration.json` — Registration receipt with DID
- `evidence/s01/bob_registration.json` — Registration receipt with DID
- `evidence/s01/channel_creation.json` — Channel with both members
- `evidence/s01/kolme_block_{height}.json` — Registration anchor

**Pass Criteria:**
- Both agents register successfully
- Channel creation succeeds (implies mutual discovery)
- Discovery completes within 10 seconds

### 7.3 S-02: Direct Message Round-Trip

**Purpose:** Validate the complete message lifecycle from creation through Kolme proof anchoring.

**Steps:**
1. Alice: `kamn_create_channel` (direct, with Bob)
2. Alice: `kamn_send_message` to Bob: "Hello Bob, E2E test {run_id}"
3. Bob: `kamn_list_messages` on the channel — reads decrypted message
4. Alice: `kamn_query_message` — lifecycle trace through Validated
5. Alice: `kamn_verify_proof` — confirms Kolme finality

**Verifiable Outputs:**
- `evidence/s02/message_send_receipt.json` — message_id + initial status
- `evidence/s02/message_lifecycle_trace.json` — Created → Signed → Broadcast → Delivered → Validated
- `evidence/s02/bob_received_messages.json` — Decrypted content (must match)
- `evidence/s02/kolme_proof_receipt.json` — tx hash + block height + finality

**Pass Criteria:**
- Message status reaches Validated within 30 seconds
- Bob's decrypted content matches Alice's plaintext exactly
- Kolme proof has finality FINAL

### 7.4 S-03: Group Channel Messaging

**Purpose:** Validate group channel creation, membership management, and multi-recipient delivery.

**Steps:**
1. Alice: `kamn_create_channel` (group, members: [Bob, Carol], name: "e2e-group-{run_id}")
2. Alice: `kamn_send_message` in group: "Group test {run_id}"
3. Bob: `kamn_list_messages` — receives message
4. Carol: `kamn_list_messages` — receives message
5. Bob: `kamn_send_message` reply in group
6. Alice + Carol: `kamn_list_messages` — both receive Bob's reply
7. (Orchestrator removes Carol via API — membership change)
8. Alice: `kamn_send_message` — Carol must NOT receive it

**Verifiable Outputs:**
- `evidence/s03/channel_create_receipt.json` — Channel with 3 members
- `evidence/s03/group_deliveries.json` — All delivery receipts
- `evidence/s03/membership_change.json` — Carol removal audit
- `evidence/s03/post_removal_isolation.json` — Carol did not receive post-removal message

**Pass Criteria:**
- All members receive messages before removal
- Post-removal message delivered to Alice and Bob only

### 7.5 S-04: Task Lifecycle (Full)

**Purpose:** Validate the complete task workflow from creation through escrow settlement.

**Steps:**
1. Alice: `kamn_create_task` — "Translate EN→FR", criteria, 100 unit escrow
2. Alice: `kamn_fund_escrow` — 100 units for the task
3. Bob: `kamn_accept_task`
4. Bob: `kamn_complete_task` — evidence hash
5. Alice: `kamn_release_escrow`
6. Both: `kamn_verify_proof` on settlement

**Verifiable Outputs:**
- `evidence/s04/task_create_receipt.json` — Task ID + metadata
- `evidence/s04/task_lifecycle_trace.json` — Pending → Accepted → Completed
- `evidence/s04/escrow_fund_receipt.json` — Lock transaction
- `evidence/s04/escrow_release_receipt.json` — Settlement
- `evidence/s04/kolme_settlement_anchor.json` — Finality proof

**Pass Criteria:**
- Lifecycle: Pending → Accepted → Completed
- Escrow: Funded → Released
- Kolme finality: FINAL

### 7.6 S-05: Escrow Settlement (Dispute)

**Purpose:** Validate the full escrow lifecycle including dispute resolution.

**Steps:**
1. Alice: `kamn_fund_escrow` — 100 units
2. Alice: `kamn_release_escrow` — partial release (60 to Bob)
3. Alice: dispute remaining amount
4. Carol (mediator): resolve — 30 to Bob, 10 refund to Alice
5. All: `kamn_verify_proof` on all transitions

**Verifiable Outputs:**
- `evidence/s05/escrow_lifecycle_trace.json` — Funded → PartiallyReleased → Disputed → Resolved
- `evidence/s05/settlement_breakdown.json` — Final balances
- `evidence/s05/dispute_resolution.json` — Carol's resolution receipt
- `evidence/s05/kolme_escrow_anchors.json` — All transitions anchored

**Pass Criteria:**
- Final balances match expected distribution
- All transitions have Kolme anchors
- Mediator authorization required for dispute resolution

### 7.7 S-06: Kolme Proof Verification

**Purpose:** Independently verify all KAMN proofs are correctly anchored.

**Steps:**
1. Collect all message_ids from S-01 through S-05
2. For each: `kamn_verify_proof`
3. Independently query Kolme `/block/{height}` for each anchor
4. Verify block hash chain continuity from genesis
5. Verify processor signatures

**Verifiable Outputs:**
- `evidence/s06/kolme_chain_audit.json` — Full chain from genesis
- `evidence/s06/proof_inclusion_map.json` — operation → block → state root
- `evidence/s06/chain_integrity_report.json` — Hash chain verification
- `evidence/s06/processor_signatures.json` — All signatures verified

**Pass Criteria:**
- Every KAMN proof has a corresponding Kolme block
- Hash chain unbroken from genesis
- All processor signatures verify
- No orphan proofs

### 7.8 S-08: Node Crash Recovery

**Purpose:** Validate KAMN nodes recover from crashes without data loss.

**Steps:**
1. Alice: send 10 messages to Bob via `kamn_send_message`
2. Verify all 10 delivered via `kamn_query_message`
3. Orchestrator: SIGKILL the KAMN processor node
4. Orchestrator: restart the processor
5. Alice: send 5 more messages
6. Verify messages 11-15 delivered correctly
7. Verify nonce continuity, no duplicates, no gaps

**Verifiable Outputs:**
- `evidence/s08/pre_crash_messages.json` — 10 delivery receipts
- `evidence/s08/crash_event.json` — PID + timestamp
- `evidence/s08/recovery_log.json` — Restart state recovery
- `evidence/s08/post_crash_messages.json` — 5 delivery receipts
- `evidence/s08/state_consistency.json` — No gaps or duplicates

**Pass Criteria:**
- Zero message loss
- Nonce sequence continuous across crash
- Processor resumes from correct block height

---

## 8. Evidence Bundle Specification

### 8.1 Evidence Directory Structure

```
evidence/
  run-{timestamp}-{run_id}/
    manifest.json              # Run metadata, scenario list, pass/fail
    infrastructure/
      kolme_genesis.json       # Kolme genesis block
      kamn_node_configs/       # All node configurations
      agent_keys/              # Public keys only (private keys ephemeral)
      mcp_server_configs/      # kamn-mcp-server launch args per agent
    s01-agent-discovery/
      alice_registration.json
      bob_registration.json
      channel_creation.json
      kolme_block_{height}.json
      scenario_result.json
    s02-direct-message/
      message_send_receipt.json
      message_lifecycle_trace.json
      bob_received_messages.json
      kolme_proof_receipt.json
      scenario_result.json
    ...
    summary_report.json
    kolme_chain_dump.json
    kamn_state_snapshots/
```

### 8.2 Manifest Schema

```json
{
  "schema_version": "kamn.e2e.evidence-manifest.v3",
  "run_id": "e2e-20260221-143052-a7b3c",
  "started_at": "2026-02-21T14:30:52Z",
  "completed_at": "2026-02-21T14:35:12Z",
  "duration_seconds": 260,
  "execution_mode": "sdk-direct",
  "infrastructure": {
    "kolme_version": "0.x.y",
    "kamn_version": "0.1.0",
    "kamn_commit": "49efe252",
    "kamn_agent_lib_version": "0.1.0",
    "agent_runtime": "sdk-direct",
    "node_count": 3,
    "agent_count": 3,
    "storage_backend": "sqlite+postgres"
  },
  "scenarios": [
    {
      "id": "S-01",
      "name": "Agent Discovery & Identity",
      "status": "PASS",
      "duration_seconds": 12,
      "evidence_files": ["s01-agent-discovery/*.json"],
      "verifiable_outputs": 4
    }
  ],
  "summary": {
    "total_scenarios": 15,
    "passed": 13,
    "failed": 1,
    "skipped": 1,
    "kolme_blocks_produced": 47,
    "messages_exchanged": 128,
    "proofs_anchored": 47,
    "proofs_verified": 47
  }
}
```

### 8.3 Verification Contract

Every evidence file includes a `_verification` block:

```json
{
  "data": { "..." : "..." },
  "_verification": {
    "evidence_hash": "sha256:abc123...",
    "captured_at": "2026-02-21T14:31:05Z",
    "source_node": "kamn-processor-1",
    "agent": "alice",
    "kolme_anchor": {
      "tx_hash": "sha256:def456...",
      "block_height": 42,
      "finality": "FINAL"
    }
  }
}
```

---

## 9. Independent Verification

A standalone verification script validates an evidence bundle offline:

```bash
kamn-e2e-verify \
  --evidence-dir evidence/run-20260221-143052-a7b3c/ \
  --kolme-chain-dump evidence/run-20260221-143052-a7b3c/kolme_chain_dump.json \
  --output verification-report.json
```

This script:
1. Verifies all signature chains in evidence files
2. Verifies Kolme block hash chain integrity
3. Verifies all KAMN proofs exist in the Kolme chain
4. Verifies message content hashes match across sender/receiver evidence
5. Produces a self-contained verification report

---

## 10. Kolme Devnet Configuration

### 10.1 Local Devnet Setup

```bash
kolme-node \
  --storage inmemory \
  --api-port 3000 \
  --processor-key $KOLME_PROCESSOR_KEY \
  --enable-notifications
```

### 10.2 KAMN Kolme Client Configuration

```bash
export KAMN_KOLME_LIVE_API_URL=http://localhost:3000
export KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary
export KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX=$KOLME_SIGNER_KEY
export KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=1
export KAMN_KOLME_LIVE_WS_URL=ws://localhost:3000/notifications
```

### 10.3 Genesis Configuration

```json
{
  "chain_name": "kamn-e2e-devnet",
  "chain_version": 1,
  "processor_pubkey": "<generated>",
  "initial_accounts": [
    { "pubkey": "<kamn-signer-pubkey>", "balance": 1000000 }
  ]
}
```

---

## 11. Orchestrator Design

### 11.1 Harness Binary

```rust
// crates/kamn-e2e-harness/src/main.rs
// Phases:
// 1. INFRA_UP      — Start PostgreSQL, Kolme devnet, KAMN nodes
// 2. AGENT_DEPLOY  — Generate keys, spawn kamn-mcp-server per agent (if MCP mode)
// 3. SCENARIO_RUN  — Execute scenarios via sdk-direct, cli, or MCP agent
// 4. EVIDENCE      — Collect and verify evidence bundles
// 5. TEARDOWN      — Stop all processes, archive evidence
```

### 11.2 Infrastructure Lifecycle

```
Phase 1: INFRA_UP (estimated: 30 seconds)
├── Start PostgreSQL container (docker)
├── Run Kolme migrations
├── Start Kolme processor (in-memory or Fjall storage)
├── Verify Kolme API health (/healthz)
├── Start KAMN processor node
├── Start KAMN listener node
├── Start KAMN approver node
├── Wait for peer discovery (3 connected peers)
└── Verify KAMN Service API health (/healthz)

Phase 2: AGENT_DEPLOY (estimated: 10 seconds)
├── Generate ed25519 key pairs for Alice, Bob, Carol
├── Write key files to temp directory
├── Register agents via kamn-agent-lib (POST /v1/agents/bootstrap)
├── [MCP modes] Spawn kamn-mcp-server per agent with identity
├── [MCP modes] Verify MCP server health
└── Record infrastructure evidence

Phase 3: SCENARIO_RUN (estimated: varies by mode)
├── For each scenario in priority order:
│   ├── Initialize scenario context
│   ├── Execute steps via configured driver:
│   │   ├── sdk-direct: Rust function calls via kamn-agent-lib
│   │   ├── cli-scripted: Shell commands via kamn-cli
│   │   └── mcp-*: Agent tool calls via MCP server
│   ├── Collect evidence from each step
│   ├── Verify pass criteria
│   └── Record scenario result
└── Aggregate results

Phase 4: EVIDENCE (estimated: 30 seconds)
├── Dump Kolme chain state
├── Dump KAMN node state snapshots
├── Verify all proof anchors independently
├── Generate chain-of-custody report
├── Compute evidence bundle hash
└── Write manifest.json

Phase 5: TEARDOWN
├── [MCP modes] Stop kamn-mcp-server processes
├── Stop KAMN nodes (graceful shutdown)
├── Stop Kolme devnet
├── Stop PostgreSQL container
└── Archive evidence bundle
```

---

## 12. CI Integration

### 12.1 SDK-Direct Mode (Weekly)

```yaml
# .github/workflows/e2e-live.yml
name: E2E Live Tests
on:
  schedule:
    - cron: '0 6 * * 1'  # Weekly Monday 6 AM UTC
  workflow_dispatch:

jobs:
  e2e-sdk-direct:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        ports: ['5432:5432']
    steps:
      - uses: actions/checkout@v4

      - name: Build KAMN (all crates including kamn-agent-lib)
        run: cargo build --workspace --release

      - name: Build Kolme devnet
        run: |
          git clone https://github.com/fpco/kolme /tmp/kolme
          cd /tmp/kolme && cargo build --release -p kolme-cli

      - name: Run E2E harness (SDK-direct mode)
        run: |
          cargo run --release -p kamn-e2e-harness -- \
            --mode sdk-direct \
            --kolme-binary /tmp/kolme/target/release/kolme-node \
            --evidence-dir /tmp/evidence \
            --scenarios S-01,S-02,S-03,S-04,S-05,S-06

      - name: Verify evidence bundle
        run: |
          cargo run --release -p kamn-e2e-harness -- verify \
            --evidence-dir /tmp/evidence/run-*/ \
            --strict

      - name: Upload evidence
        uses: actions/upload-artifact@v4
        with:
          name: e2e-evidence-${{ github.sha }}
          path: /tmp/evidence/
```

### 12.2 MCP Agent Mode (Nightly)

```yaml
  e2e-mcp-agent:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4

      - name: Build KAMN
        run: cargo build --workspace --release

      - name: Build agent runtime (Tau)
        run: |
          git clone https://github.com/njfio/Tau /tmp/tau
          cd /tmp/tau && cargo build --release

      - name: Run E2E harness (MCP-Tau mode)
        env:
          ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
        run: |
          cargo run --release -p kamn-e2e-harness -- \
            --mode mcp-tau \
            --agent-binary /tmp/tau/target/release/tau \
            --kolme-binary /tmp/kolme/target/release/kolme-node \
            --evidence-dir /tmp/evidence \
            --scenarios all
```

### 12.3 CLI-Scripted Mode (Per-Commit Smoke)

```yaml
  e2e-cli-smoke:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build KAMN
        run: cargo build --workspace --release
      - name: Run E2E smoke (CLI mode, P0 scenarios only)
        run: |
          cargo run --release -p kamn-e2e-harness -- \
            --mode cli-scripted \
            --kolme-binary /tmp/kolme/target/release/kolme-node \
            --evidence-dir /tmp/evidence \
            --scenarios S-01,S-02
```

---

## 13. Repository Structure

All new crates live in the KAMN repository. No code changes needed in Tau or any other agent runtime.

```
crates/
  kamn-agent-lib/                    # Shared core library
    Cargo.toml
    src/
      lib.rs                         # Public API: KamnAgentHandle
      identity.rs                    # DID generation, key management, key derivation
      auth.rs                        # KAMN auth header construction + signatures
      envelope.rs                    # CanonicalMessageEnvelope builder + encryption
      client.rs                      # ServiceApiHttpClient (typed HTTP client)
      kolme.rs                       # Kolme proof fetching + verification
      nonce.rs                       # Monotonic nonce tracking
      errors.rs                      # Error taxonomy
    tests/
      auth_roundtrip.rs              # Signature → KAMN server → 200 OK
      envelope_construction.rs       # Envelope → decrypt → match
      kolme_verification.rs          # Proof chain validation

  kamn-mcp-server/                   # MCP tool server binary
    Cargo.toml
    src/
      main.rs                        # CLI args + MCP server bootstrap
      tools.rs                       # 12 MCP tool registrations
      config.rs                      # Agent identity + endpoint config

  kamn-cli/                          # Command-line interface binary
    Cargo.toml
    src/
      main.rs                        # CLI args (clap) + subcommand dispatch
      commands/
        register.rs
        send_message.rs
        create_channel.rs
        list_messages.rs
        query_message.rs
        create_task.rs
        accept_task.rs
        complete_task.rs
        fund_escrow.rs
        release_escrow.rs
        verify_proof.rs
        health.rs

  kamn-e2e-harness/                  # E2E test orchestrator binary
    Cargo.toml
    src/
      main.rs                        # CLI entry point with mode selection
      infrastructure.rs              # Docker + process lifecycle management
      kolme_devnet.rs                # Kolme setup + genesis
      identity.rs                    # Key generation for test agents
      drivers/
        mod.rs
        sdk_direct.rs                # Direct kamn-agent-lib calls
        cli_scripted.rs              # Shell kamn-cli commands
        mcp_agent.rs                 # MCP server spawn + agent coordination
      scenarios/
        mod.rs
        s01_discovery.rs
        s02_message.rs
        s03_group.rs
        s04_task.rs
        s05_escrow.rs
        s06_kolme_verify.rs
        s08_crash_recovery.rs
      evidence.rs                    # Evidence bundle writer + manifest
      verify.rs                      # Offline verification logic

docs/
  skills/
    kamn-agent.md                    # Portable skill definition for any LLM agent
```

---

## 14. Implementation Plan

### Phase 1: kamn-agent-lib (3-4 weeks)

The shared library is the foundation. Everything else depends on it.

| Task | Deliverable | Effort |
|------|------------|--------|
| Identity module (DID generation, key management, key derivation) | `AgentIdentity` with ed25519 + X25519 | M |
| Auth module (signature format, header construction) | `KamnAuthHeaders::build()` | M |
| Envelope module (construction, encryption, signing) | `build_and_sign_envelope()` | L |
| Service API HTTP client (all 11 routes) | `ServiceApiHttpClient` | L |
| Kolme proof verification client | `KolmeClient::verify_proof()` | M |
| Nonce tracking + error taxonomy | Monotonic counter + `AgentLibError` enum | S |
| `KamnAgentHandle` top-level API | Ergonomic facade over all modules | M |
| Integration tests against running KAMN node | Auth roundtrip, envelope decrypt, proof chain | M |

### Phase 2: MCP Server + CLI (2-3 weeks)

Thin wrappers over `kamn-agent-lib`. Can be developed in parallel.

| Task | Deliverable | Effort |
|------|------------|--------|
| `kamn-mcp-server` binary scaffold | CLI args, MCP stdio transport, config | M |
| 12 MCP tool definitions with JSON schemas | All tools registered and documented | L |
| MCP integration tests (tool call → KAMN API → response) | End-to-end MCP validation | M |
| `kamn-cli` binary scaffold | Clap CLI with 12 subcommands | M |
| CLI JSON + text output modes | `--format json` / `--format text` | S |
| CLI environment variable support | `KAMN_ENDPOINT`, `KAMN_AGENT_DID`, etc. | S |
| Agent skill definition (`docs/skills/kamn-agent.md`) | Portable prompt for any LLM | S |

### Phase 3: E2E Harness + Core Scenarios (3-4 weeks)

| Task | Deliverable | Effort |
|------|------------|--------|
| `kamn-e2e-harness` binary scaffold with 4 execution modes | Mode selection + CLI args | M |
| Infrastructure lifecycle (INFRA_UP / TEARDOWN) | Docker + process management | M |
| Kolme devnet bootstrap | Genesis + health check | S |
| KAMN multi-node startup | 3-node topology with peer discovery | M |
| Evidence bundle writer + manifest | JSON evidence + verification blocks | M |
| SDK-direct driver | Direct `kamn-agent-lib` calls | S |
| CLI-scripted driver | Shell `kamn-cli` commands | S |
| MCP agent driver | `kamn-mcp-server` spawn + agent coordination | M |
| S-01 Agent Discovery | First passing scenario | M |
| S-02 Direct Message Round-Trip | Full lifecycle validation | L |
| S-03 Group Channel | Membership + multi-recipient | M |
| S-04 Task Lifecycle | Task state machine + escrow | L |
| S-06 Kolme Proof Verification | Independent chain audit | M |

### Phase 4: Remaining Scenarios + CI (2-3 weeks)

| Task | Deliverable | Effort |
|------|------------|--------|
| S-05 Escrow Settlement (dispute) | 3-agent dispute resolution | L |
| S-08 Crash Recovery | SIGKILL + restart + state verification | M |
| S-09 Transport Failover | libp2p disconnect + reconnect | M |
| Evidence verification script | Offline bundle validation | M |
| CI: SDK-direct weekly pipeline | GitHub Actions | S |
| CI: MCP-agent nightly pipeline | With Tau + API key | S |
| CI: CLI smoke per-commit pipeline | P0 scenarios only | S |
| Documentation + runbook | Developer guide for all modes | S |

### Effort Key: S = 1-2 days, M = 3-5 days, L = 5-8 days

**Total estimated effort: 10-14 weeks**

Phase 1 is the critical path. Phases 2 and 3 can partially overlap once the core `kamn-agent-lib` modules stabilize.

---

## 15. Success Metrics

| Metric | Target |
|--------|--------|
| Scenarios passing (SDK-direct) | 15/15 (100%) |
| Scenarios passing (CLI-scripted) | 15/15 (100%) |
| Scenarios passing (MCP agent) | 13/15 (87%) — allowing for LLM non-determinism |
| Evidence bundle size | < 50 MB per run |
| Full run (SDK-direct) | < 2 minutes |
| Full run (CLI-scripted) | < 3 minutes |
| Full run (MCP agent) | < 15 minutes |
| Kolme blocks produced per run | 30-60 |
| Independent verification pass rate | 100% |
| CI weekly pass rate | > 99% |
| CI nightly pass rate | > 90% |
| Agent runtimes validated | >= 2 (Tau + one other) |

---

## 16. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLM agent non-determinism | Flaky nightly tests | SDK-direct as primary CI gate; MCP agent mode is advisory |
| KAMN auth signature format complexity | Integration bugs | Dedicated test suite against known KAMN test vectors |
| MCP protocol version drift | Server incompatibility | Pin MCP SDK version; test against multiple clients |
| Kolme devnet startup latency | Slow runs | In-memory storage; pre-compiled binary caching |
| libp2p peer discovery timeout in CI | False failures | Configurable timeout; fallback to direct TCP |
| kamn-agent-lib maintenance burden | Library drifts from Service API | Integration tests run against live KAMN node in CI |
| Port conflicts in CI | Infrastructure failures | Dynamic port allocation |
| Evidence bundle disk usage | Storage costs | 30-day retention; compressed archives |
| LLM API key costs for agent mode | Budget | Haiku model; weekly not daily |

---

## 17. Open Questions

1. **MCP transport**: stdio (simplest, one server per agent) or SSE (shared server, multi-agent)? Stdio is simpler for E2E testing; SSE is better for interactive use.
2. **Kolme devnet from source**: Build from source or maintain a Docker image?
3. **Agent model selection**: Which LLM model for MCP agent mode? Haiku for cost, Sonnet for reliability?
4. **Multi-host testing**: Localhost with different ports sufficient for v1?
5. **Kolme chain persistence**: Fresh per run or persistent across runs?
6. **kamn-agent-lib publication**: Publish to crates.io for external consumption, or workspace-only for now?
7. **Key management for production**: Key files, environment variables, or a secret manager integration?

---

## 18. Appendix: Subsystem Coverage Map

| KAMN Module | Covered By Scenarios | Coverage Level |
|-------------|---------------------|----------------|
| `did` / `did_registry` | S-01 | Full |
| `agent_key_hierarchy` | S-01, S-11 | Full |
| `message_envelope` | S-02, S-03 | Full |
| `message_lifecycle` | S-02, S-03, S-07 | Full |
| `direct_message_crypto` | S-02 | Full |
| `group_channel_crypto` | S-03 | Full |
| `message_delivery_guards` | S-07 | Full |
| `message_proof_anchoring` | S-02, S-06, S-14 | Full |
| `channel_models` / `channel_policies` | S-03, S-12 | Full |
| `task_operations` / `task_lifecycle` | S-04 | Full |
| `escrow` / `task_payment` | S-04, S-05 | Full |
| `p2p_transport` (libp2p) | S-01, S-09, S-10 | Full |
| `kolme_runtime_commit` | S-06, S-14 | Full |
| `sqlite_store_backend` | S-08 | Partial |
| `data_layer_postgres_*` | S-10 | Partial |
| `data_layer_m1_*` (Merkle batching) | S-14 | Full |
| `runtime` (daemon lifecycle) | S-08, S-09 | Full |
| `bootstrap` / `config` | All scenarios | Full |
| `signer_backend` | S-02, S-11 | Full |
| `content_lifecycle` / `retention_engine` | S-12 | Full |
| `redaction_compliance` | S-12 | Partial |
| `bridge_adapter` | S-13 | Partial |
| `service_marketplace` | — | Not covered |
| `observability` | S-15 | Partial |
| `trust_score` / `reputation_*` | — | Not covered |
| `cross_store_replay_consistency` | S-10 | Partial |
| `dependency_ci_smoke_policy` | — | Not covered (CI-only) |

---

## 19. Appendix: KAMN Authentication Reference

For implementers of `kamn-agent-lib` and for understanding the MCP server internals.

### 19.1 Signature Format

```
sig:ed25519:baseline-v1:{sender_did}:{nonce}:{state_hash}:{body_len}
```

- `{sender_did}`: KAMN DID (`kamn:did:agent:{name}`)
- `{nonce}`: Monotonically increasing u64 per agent
- `{state_hash}`: SHA-256 of current agent state (latest committed block hash)
- `{body_len}`: Byte length of the HTTP request body

### 19.2 Required HTTP Headers

| Header | Value | Purpose |
|--------|-------|---------|
| `x-kamn-sender-did` | `kamn:did:agent:{name}` | Agent identity |
| `x-kamn-request-nonce` | `{u64}` | Replay protection |
| `x-kamn-request-signature` | Base64(ed25519-sign(sig-input)) | Authentication |
| `x-kamn-authz-scope` | Scope string (optional) | Authorization |

### 19.3 Service API Routes

| Route | Method | Auth Required | Purpose |
|-------|--------|--------------|---------|
| `/v1/agents/bootstrap` | POST | No | Register new agent DID |
| `/v1/messages/send` | POST | Yes | Send encrypted message |
| `/v1/messages/{id}` | GET | Yes | Query message status |
| `/v1/channels/create` | POST | Yes | Create channel |
| `/v1/channels/{id}` | GET | Yes | Query channel |
| `/v1/channels/{id}/messages` | GET | Yes | List channel messages |
| `/v1/tasks/create` | POST | Yes | Create task |
| `/v1/tasks/{id}` | GET | Yes | Query task |
| `/v1/tasks/{id}/accept` | POST | Yes | Accept task |
| `/v1/tasks/{id}/complete` | POST | Yes | Complete task |
| `/v1/escrow/fund` | POST | Yes | Fund escrow |
| `/v1/escrow/{id}/release` | POST | Yes | Release escrow |
| `/v1/events/ws` | WebSocket | Yes | Real-time events |
| `/healthz` | GET | No | Health check |
| `/metrics` | GET | No | Metrics |
