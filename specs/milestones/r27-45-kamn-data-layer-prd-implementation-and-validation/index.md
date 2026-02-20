# R27.45 KAMN data layer PRD implementation and validation

## Milestone Summary
Execution milestone for `docs/planning/kamn-data-layer-prd.docx.md` covering full PRD delivery from M0 through M11. The objective is to implement, integrate, test, and validate a PostgreSQL-centric privacy-first data layer with Kolme trust anchoring, deterministic crypto and integrity proofs, intelligence layers (vector + graph + time-series), compliance lifecycle controls, and production hardening.

## Source Artifacts
- PRD source: `docs/planning/kamn-data-layer-prd.docx.md`
- Master execution plan: `docs/plans/2026-02-18-kamn-data-layer-prd-execution-plan.md`
- Infrastructure activation plan: `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`

## Issue Hierarchy
- Epic:
  - `#5002` — Epic: execute KAMN data layer PRD M0-M11 with full integration and validation
  - `#5247` — Epic: activate data-layer infrastructure layer and close contract-to-runtime gap
- Stories:
  - `#5003` — Story: M0 foundation schema, append-only enforcement, and envelope crypto pipeline
  - `#5004` — Story: M1 trust anchor merkle batching and Kolme anchoring integration
  - `#5005` — Story: M2 DID access gateway with ABAC, RLS, and audit trails
  - `#5006` — Story: M3 metadata and blind-index search surfaces
  - `#5007` — Story: M4 escrow-scoped messaging and settlement evidence integration
  - `#5008` — Story: M5 vector embedding layer and semantic search
  - `#5009` — Story: M6 knowledge graph layer and trust propagation queries
  - `#5010` — Story: M7 time-series telemetry, aggregates, and billing metrics
  - `#5011` — Story: M8 compliance lifecycle with crypto-shredding and retention controls
  - `#5012` — Story: M9 real-time delivery, presence, and flow control
  - `#5013` — Story: M10 scaling, partition management, and archival pipelines
  - `#5014` — Story: M11 hardening with security, chaos, and benchmark validation
  - `#5015` — Story: cross-cutting conformance harness and shell-surface budget neutrality
  - `#5248` — Story: build PostgreSQL persistence foundation and RLS contract bridge
  - `#5249` — Story: implement envelope crypto and blind-index operational pipeline
  - `#5250` — Story: wire Kolme anchoring client and merkle batch scheduler
  - `#5251` — Story: deliver extension-backed intelligence adapters (pgvector, AGE, TimescaleDB)
  - `#5252` — Story: implement realtime delivery gateway and owner-scoped presence service
  - `#5253` — Story: automate retention crypto-shredding and partition archival execution
  - `#5254` — Story: enforce contract-infrastructure convergence and operational failure-path validation
- Tasks:
  - `#5016` — Task: M0 deliver core schema, append-only controls, and envelope crypto primitives
  - `#5017` — Task: M1 implement merkle batching, Kolme anchoring worker, and proof APIs
  - `#5018` — Task: M2 ship DID gateway authn/authz, RLS policy set, and audit log path
  - `#5019` — Task: M3 implement blind-index + metadata search APIs with deterministic tests
  - `#5020` — Task: M4 integrate escrow state, scoped messaging, and settlement evidence storage
  - `#5021` — Task: M5 deliver pgvector embeddings pipeline and semantic query endpoints
  - `#5022` — Task: M6 deliver Apache AGE graph schema and trust propagation query service
  - `#5023` — Task: M7 deliver Timescale hypertables, aggregates, and billing telemetry surfaces
  - `#5024` — Task: M8 deliver crypto-shredding, retention policy enforcement, and legal-hold controls
  - `#5025` — Task: M9 deliver realtime delivery pipeline, presence, and deterministic backpressure
  - `#5026` — Task: M10 deliver scaling controls, partition lifecycle, and archival export path
  - `#5027` — Task: M11 execute hardening matrix (security, chaos, perf) and operator readiness
  - `#5028` — Task: enforce PRD critical-scenario conformance matrix with shell-neutral test orchestration
  - `#5255` — Task: bootstrap data-layer PostgreSQL migration scaffolding and schema contract markers
  - `#5257` — Task: implement PostgreSQL repository bridge contracts and RLS session projection
  - `#5259` — Task: implement sqlx-backed PostgreSQL execution adapter and migration runner
  - `#5261` — Task: extend PostgreSQL execution adapter with RLS policy application and blind-index search execution
  - `#5263` — Task: implement Phase-2 envelope+blind-index operational pipeline and persist blind indexes in adapter inserts
  - `#5265` — Task: implement M1 batch scheduler thresholds and merkle-batch persistence execution path
  - `#5267` — Task: implement M1 anchoring orchestrator tick and persistence-plan projection
  - `#5269` — Task: implement M1 anchoring follow-up retry and confirmation policy projection
  - `#5271` — Task: implement M1 finality-observation reconciliation projection
  - `#5273` — Task: implement M5 pgvector adapter projection and fail-closed extension contracts
  - `#5275` — Task: implement M6 AGE adapter projection and fail-closed graph-extension contracts
  - `#5277` — Task: implement M7 Timescale adapter projection and fail-closed telemetry-extension contracts
  - `#5279` — Task: implement M9 realtime gateway bridge projection and fail-closed presence-scope contracts
  - `#5281` — Task: integrate service-api websocket presence route with M9 gateway bridge contracts
  - `#5283` — Task: validate realtime backpressure guardrails and finalize presence gateway ops docs
  - `#5285` — Task: start Phase-6 retention-to-archival gate execution contracts
  - `#5287` — Task: add deterministic archival failure-retry policy contracts for Phase-6
  - `#5289` — Task: add Phase-6 retention+archival execution tick orchestration contracts
  - `#5291` — Task: add Phase-6 execution-tick budget guardrail contracts
  - `#5293` — Task: add Phase-6 scheduler-cycle trigger and guarded execution contracts
  - `#5295` — Task: add stateful Phase-6 scheduler runtime checkpoint contracts
  - `#5297` — Task: add Phase-6 runtime evidence bundle projection contracts
- Subtasks:
  - `#5029` — Subtask: M0 conformance matrix for envelope crypto, append-only, and hash-chain invariants
  - `#5030` — Subtask: M1 deterministic merkle proof and Kolme anchoring failure-matrix coverage
  - `#5031` — Subtask: M2 DID auth + ABAC + RLS negative matrix and audit evidence fixtures
  - `#5032` — Subtask: M3 blind-index correctness and search determinism regression corpus
  - `#5033` — Subtask: M4 escrow message visibility and settlement evidence integrity contracts
  - `#5034` — Subtask: M5 vector recall, drift, and anomaly-score regression harness
  - `#5035` — Subtask: M6 graph trust-propagation correctness and portability boundary contracts
  - `#5036` — Subtask: M7 telemetry aggregate correctness and billing reconciliation regressions
  - `#5037` — Subtask: M8 crypto-shred and retention-policy legal-hold conformance suite
  - `#5038` — Subtask: M9 realtime delivery ordering, presence, and backpressure fail-closed checks
  - `#5039` — Subtask: M10 partition lifecycle and archival recoverability contract suite
  - `#5040` — Subtask: M11 security-chaos-performance closure evidence and acceptance report
  - `#5041` — Subtask: shell-neutral test orchestration guardrail and ratio-budget evidence policy

## Governance Markers
- `shell_loc_hard_ceiling_env=.ci/shell-loc-hard-ceiling.env`
- `shell_rust_ratio_guardrail_env=.ci/shell-rust-ratio-guardrail.env`
- `shell_loc_hard_ceiling_max=130000`
- `warn_shell_rust_ratio_max=0.95`
- `fail_shell_rust_ratio_max=1.00`
- `default_test_orchestration_mode=rust-first`
- `shell_surface_waiver_policy=required_for_any_net_shell_loc_growth`
