# Issue #5105 Spec

- Title: Task: integrate M5 vector contracts with AgentDid parsing and content lifecycle retention
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
M5 currently validates DID fields with local string checks and has no retention bridge to `content_lifecycle`. This leaves vector identity and retention behavior detached from canonical runtime contracts.

## Acceptance Criteria
- AC-1: M5 validates `agent_did` through `AgentDid::parse`.
- AC-2: M5 exposes owner-scoped retention-due projection aligned to `ContentLifecycleManager` retention windows.
- AC-3: Invalid agent DID and invalid retention projection inputs fail closed with deterministic M5 taxonomy.
- AC-4: Existing semantic query, anomaly detection, recall drift, and hash-chain behavior remains backward compatible.
- AC-5: Shell/workflow/python/template LOC remain unchanged (`shell_loc_delta_actual = 0`).

## Scope
In scope:
- `crates/kamn-core/src/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/tests/data_layer_m5_vector_integration.rs`
- `crates/kamn-core/src/lib.rs`
- `specs/5105/{spec.md,plan.md,tasks.md}`

Out of scope:
- Storage schema migrations.
- Runtime/deployment changes.
- Dependency changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | M5 append/anomaly inputs with invalid `agent_did` format | Fail-closed deterministic `InvalidAgentDid` taxonomy |
| C-02 | AC-2 | Functional | Owner records with mixed retention classes and now timestamp | Deterministic retention-due list using content lifecycle windows |
| C-03 | AC-3 | Regression | Invalid retention projection input (for example zero now timestamp) | Deterministic fail-closed taxonomy |
| C-04 | AC-4 | Regression | Existing M5 `spec_c01..spec_c09` suite | Existing behavior remains green |
| C-05 | AC-5 | Regression | Shell guardrails | Zero shell delta; guardrails GO |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m5_vector_integration`
- `cargo test -p kamn-core`
- `cargo fmt --check`
- `cargo clippy -p kamn-core -- -D warnings`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail-5105.json`
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling-5105.json`

## Success Metrics
- M5 identity validation reuses canonical DID parsing.
- M5 retention projection is aligned with content lifecycle retention windows.
- Shell governance posture is unchanged or improved.
