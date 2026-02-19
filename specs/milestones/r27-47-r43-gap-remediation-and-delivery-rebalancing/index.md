# R27.47 R43 gap remediation and delivery rebalancing

## Milestone Summary
Close the remaining R43 concerns by decomposing the new top structural monolith (`service_api_endpoint.rs`), continuing governance-surface reductions (doc-contract and shell-test migration follow-through), and documenting data-layer integration design decisions.

## Source Artifact
- Review source: `docs/review/gaps-and-issues-r43.md`

## Issue Hierarchy
- Epic:
  - `#5213` — Epic: R43 close structural maintainability and delivery-balance gaps
- Stories:
  - `#5214` — Story: decompose `service_api_endpoint.rs` with route/auth/websocket parity guarantees
  - `#5216` — Story: continue governance-surface reduction and delivery-balance controls
  - `#5220` — Story: codify data-layer standalone-module decision and integration backlog markers
- Tasks:
  - `#5215` — Task: decompose `service_api_endpoint.rs` into focused modules with parity contracts
  - `#5217` — Task: continue doc-contract harness consolidation below 100 suite files
  - `#5218` — Task: execute shell-test migration wave 2 and maintain shell:rust ratio improvement
  - `#5219` — Task: add governance-to-feature activity telemetry and release review markers
  - `#5221` — Task: document M11/PRD standalone status and typed-DID migration backlog markers

## Governance Markers
- `shell_loc_hard_ceiling_env=.ci/shell-loc-hard-ceiling.env`
- `shell_rust_ratio_guardrail_env=.ci/shell-rust-ratio-guardrail.env`
- `service_api_endpoint_root_target_line_budget=900`
- `doc_contract_suite_target_max_files=100`
