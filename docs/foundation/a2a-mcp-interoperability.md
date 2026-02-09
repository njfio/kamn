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

## A2A/MCP Conformance Harness Evidence Contract (Issue #893)
A2A/MCP request-response interoperability must run through deterministic fixture replay and fail closed on mapping drift before CI contracts return `GO`.

- Conformance harness runner:
  - `python3 scripts/message/run_a2a_mcp_conformance_harness.py --fixture fixtures/a2a_mcp_conformance/replay_cases.json --output-json /tmp/a2a-mcp-conformance-report.json`
- Policy checker:
  - `bash scripts/message/check_a2a_mcp_conformance_policy.sh --report-file /tmp/a2a-mcp-conformance-report.json`
- PR fast contract lane:
  - `bash scripts/message/run_a2a_mcp_conformance_contract_lane.sh`
- Decision key contract:
  - `a2a_mcp_conformance_reason_codes:GO:v1`
- Regression policy:
  - report schema/key drift and case decision mismatches force `NO-GO` (`Regression: #893`).

## Local Validation
Run from repository root:

```bash
bash scripts/message/test_run_a2a_mcp_conformance_harness.sh
bash scripts/message/test_check_a2a_mcp_conformance_policy.sh
bash scripts/message/test_run_a2a_mcp_conformance_contract_lane.sh
cargo test -p kamn-core --test a2a_mcp_interop_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
