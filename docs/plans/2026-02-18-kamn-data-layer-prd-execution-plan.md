# 2026-02-18 KAMN Data Layer PRD Execution Plan

## Source and Objective
- Source PRD: `kamn-data-layer-prd.docx.md`
- Milestone container: `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- Objective: Execute PRD milestones M0-M11 with contract-driven implementation, integration, testing, and validation, while keeping shell-surface growth neutral by default.

## Contract Process (AGENTS.md)
- Each implementation issue must maintain `specs/<issue-id>/spec.md`, `plan.md`, and `tasks.md`.
- Lifecycle order is mandatory: `SPECIFY -> PLAN -> TASKS -> IMPLEMENT -> VERIFY`.
- TDD is mandatory per task: `Red -> Green -> Refactor -> Regression -> Verify`.
- No implementation starts without milestone + issue + spec readiness.
- Shell-surface DoR/DoD markers are mandatory for any shell/python/workflow/template touch.

## Issue Hierarchy
- Epic: `#5002`
- Stories: `#5003`..`#5015`
- Tasks: `#5016`..`#5028`
- Subtasks: `#5029`..`#5041`

## Milestone-to-Issue Mapping
| PRD Milestone | Story | Task | Subtask | Duration (PRD) | Delivery Focus |
|---|---|---|---|---|---|
| M0 Foundation | #5003 | #5016 | #5029 | 4 weeks | Schema, append-only enforcement, envelope crypto, zstd, hash-chain integrity |
| M1 Trust Anchor | #5004 | #5017 | #5030 | 3 weeks | Merkle batches, Kolme anchoring worker, proof generation/verification |
| M2 Access Gateway | #5005 | #5018 | #5031 | 4 weeks | DID auth, ABAC, RLS, rate controls, access audit trail |
| M3 Search and Indexing | #5006 | #5019 | #5032 | 2 weeks | Blind indexes, metadata/full-text query path |
| M4 Escrow Integration | #5007 | #5020 | #5033 | 4 weeks | Escrow message scope, settlement evidence, auditor access constraints |
| M5 Vector Layer | #5008 | #5021 | #5034 | 3 weeks | pgvector embeddings, semantic search, anomaly scoring |
| M6 Graph Layer | #5009 | #5022 | #5035 | 3 weeks | Apache AGE schema, trust propagation, capability matchmaking |
| M7 Time-Series | #5010 | #5023 | #5036 | 2 weeks | Timescale hypertables, aggregates, alert and billing telemetry |
| M8 Compliance | #5011 | #5024 | #5037 | 3 weeks | Crypto-shredding, retention, legal-hold, export compliance |
| M9 Real-Time | #5012 | #5025 | #5038 | 3 weeks | WebSocket/SSE delivery, presence, eventing, flow control |
| M10 Scaling | #5013 | #5026 | #5039 | 3 weeks | Partition lifecycle, read scaling, archival/export pipeline |
| M11 Hardening | #5014 | #5027 | #5040 | 4 weeks | Security audit matrix, chaos, performance, operations closure |
| Cross-cutting Validation | #5015 | #5028 | #5041 | Continuous | PRD critical scenarios + shell-neutral orchestration guardrail |

## Integration Plan
1. Foundation before integration: M0 and M1 must complete before higher-layer dependencies consume storage and anchoring APIs.
2. Security boundary first: M2 must complete before exposing search, escrow, and intelligence surfaces.
3. Functional layering: M3 and M4 build atop M0-M2 and provide business-complete MVP boundary.
4. Intelligence layering: M5-M7 integrate after M3 metadata/search surfaces are stable.
5. Policy and operations closure: M8-M11 complete compliance, runtime delivery, scale, and resilience.
6. Cross-cutting conformance issue (`#5015/#5028/#5041`) runs across all phases as merge-gate evidence.

## Testing and Validation Plan
The PRD testing categories are mapped to AGENTS test tiers and required commands.

| PRD Category | AGENTS Tier(s) | Required Commands / Evidence |
|---|---|---|
| Unit Tests | Unit | `cargo test -p kamn-core -- <module_or_test>` |
| Contract Tests | Contract/DbC + Functional | schema/policy contract suites + deterministic reason-code checks |
| Integration Tests | Integration + Conformance | end-to-end encrypt->store->query->decrypt->anchor flows |
| Crypto Verification | Property + Regression | proptest invariants + explicit regression fixtures |
| Performance Tests | Performance | criterion/pgbench benchmark artifacts with regression thresholds |
| Chaos Tests | Regression + Integration | deterministic fault-injection scenarios and outcome markers |
| Security Tests | Functional + Regression | negative-matrix authorization and key/rotation edge paths |
| Compliance Tests | Conformance + Regression | crypto-shred, retention, export completeness checks |

### Critical Scenarios (PRD 18.2)
Each scenario (62-71) is assigned to the cross-cutting validation track (`#5015/#5028/#5041`) and must have:
- one conformance case in issue `spec.md`,
- one deterministic automated test,
- one merge-gate evidence artifact.

## Shell LOC and Ratio Guardrails
To satisfy current ratio pressure, test orchestration is Rust-first and shell-neutral by default.

### Hard constraints
- `HARD_SHELL_LOC_MAX=130000` from `.ci/shell-loc-hard-ceiling.env`
- `WARN_SHELL_RUST_RATIO_MAX=0.95` and `FAIL_SHELL_RUST_RATIO_MAX=1.00` from `.ci/shell-rust-ratio-guardrail.env`

### Required checks when shell/python/workflow/template surface is touched
- `bash scripts/ci/check_shell_loc_hard_ceiling.sh --repo-root . --output-json /tmp/shell-loc-hard-ceiling.json`
- `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --repo-root . --output-json /tmp/shell-rust-ratio-guardrail.json`
- `bash scripts/ci/check_shell_surface_threshold_ratchet.sh --repo-root . --output-json /tmp/shell-surface-threshold-ratchet.json`

### Planning policy
- Default `shell_loc_delta_estimate` is `0` for all PRD execution issues.
- Any issue that requires net shell growth must:
  - add a linked mitigation issue in `shell_surface_mitigation_issue`,
  - prove equivalent or greater shell deletion in same wave, or
  - provide explicit waiver evidence and ratio-neutral justification.
- Prefer Rust tests, SQL migration tests, and existing manifest runners over new shell wrappers.

## Delivery Cadence
1. Execute one task/subtask PR at a time with complete lifecycle evidence.
2. Keep PRs bounded to a single issue unless parent/child coupling is unavoidable.
3. Publish issue process logs for each phase transition.
4. Track cumulative shell/rust deltas in closure comments for all shell-surface touching issues.

## Exit Criteria
- All stories/tasks/subtasks under milestone R27.45 are `status:done`.
- All ACs across issue specs are mapped to passing conformance tests.
- PRD critical scenarios 62-71 are fully automated and green.
- Shell hard ceiling and ratio guardrails remain compliant without unmitigated waivers.
