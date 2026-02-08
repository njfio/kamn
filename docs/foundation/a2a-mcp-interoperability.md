# A2A and MCP Interoperability Mapping (Issues #176, #177)

This document defines a deterministic mapping profile between KAMN message/task primitives and A2A/MCP-oriented integration concepts.

## Message Type Mapping
| KAMN Envelope Header | A2A Concept | MCP Concept | Notes |
|---|---|---|---|
| Request | task.invoke | tool_call | Deterministic request dispatch. |
| Response | task.result | tool_result | Deterministic completion response. |
| Event | event.notify | notification | Non-blocking external signal. |

## Task Lifecycle Mapping
| KAMN Task State | A2A Task State | MCP-Oriented Interpretation |
|---|---|---|
| Submitted | pending | Awaiting execution slot. |
| InProgress | running | Active execution in progress. |
| Blocked | paused | Execution paused on external dependency. |
| Completed | succeeded | Terminal success state. |
| Failed | failed | Terminal failure state. |
| Cancelled | cancelled | Terminal operator-initiated stop. |

## Deterministic SDK Examples
- Rust SDK mapping example:
  - `Request` envelope maps to `task.invoke` and `tool_call`.
  - `Response` envelope maps to `task.result` and `tool_result`.
- Python SDK mapping example:
  - task `Submitted` -> `pending`, task `InProgress` -> `running`.
- TypeScript SDK mapping example:
  - task `Completed` -> `succeeded`, task `Failed` -> `failed`.

## Limitations and Fallback Behavior
- Unknown external type maps to ManualReview.
- Lossy mapping paths must emit interoperability warning metadata.
- Unsupported lifecycle transition requests are rejected and routed to operator review.
- Ambiguous mapping decision: ManualReview.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test a2a_mcp_interop_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
