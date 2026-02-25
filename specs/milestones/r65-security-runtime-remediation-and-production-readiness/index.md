# Milestone R65 - Security Runtime Remediation and Production Readiness

- Milestone: `r65-security-runtime-remediation-and-production-readiness`
- Epic: #5915
- Stories: #5916, #5917, #5918, #5919, #5920
- Tasks: #5921, #5922, #5923, #5924, #5925, #5926, #5927, #5928, #5929, #5930, #5931, #5932, #5933, #5934, #5935, #5936, #5937, #5938, #5939, #5940, #5941, #5947

## Problem Frame
This milestone closes production-readiness blockers across cryptography correctness, end-to-end delivery reality, transport hardening, bounded replay protection, assurance gates, and architecture/governance sustainability.

## Execution Order (Dependency-Aware)
1. Security primitives and auth correctness: #5921, #5922, #5923, #5924, #5925
2. Real runtime delivery and bounded state: #5926, #5927, #5928
3. SDK/service transport hardening: #5929, #5930, #5931, #5932
4. Architecture and sustainability remediation: #5933, #5934, #5935, #5936, #5947
5. Assurance and CI gate expansion: #5937, #5938, #5939, #5940, #5941

## Artifact Index
- #5915: `specs/5915/spec.md`, `specs/5915/plan.md`, `specs/5915/tasks.md`
- #5916: `specs/5916/spec.md`, `specs/5916/plan.md`, `specs/5916/tasks.md`
- #5917: `specs/5917/spec.md`, `specs/5917/plan.md`, `specs/5917/tasks.md`
- #5918: `specs/5918/spec.md`, `specs/5918/plan.md`, `specs/5918/tasks.md`
- #5919: `specs/5919/spec.md`, `specs/5919/plan.md`, `specs/5919/tasks.md`
- #5920: `specs/5920/spec.md`, `specs/5920/plan.md`, `specs/5920/tasks.md`
- #5921: `specs/5921/spec.md`, `specs/5921/plan.md`, `specs/5921/tasks.md`
- #5922: `specs/5922/spec.md`, `specs/5922/plan.md`, `specs/5922/tasks.md`
- #5923: `specs/5923/spec.md`, `specs/5923/plan.md`, `specs/5923/tasks.md`
- #5924: `specs/5924/spec.md`, `specs/5924/plan.md`, `specs/5924/tasks.md`
- #5925: `specs/5925/spec.md`, `specs/5925/plan.md`, `specs/5925/tasks.md`
- #5926: `specs/5926/spec.md`, `specs/5926/plan.md`, `specs/5926/tasks.md`
- #5927: `specs/5927/spec.md`, `specs/5927/plan.md`, `specs/5927/tasks.md`
- #5928: `specs/5928/spec.md`, `specs/5928/plan.md`, `specs/5928/tasks.md`
- #5929: `specs/5929/spec.md`, `specs/5929/plan.md`, `specs/5929/tasks.md`
- #5930: `specs/5930/spec.md`, `specs/5930/plan.md`, `specs/5930/tasks.md`
- #5931: `specs/5931/spec.md`, `specs/5931/plan.md`, `specs/5931/tasks.md`
- #5932: `specs/5932/spec.md`, `specs/5932/plan.md`, `specs/5932/tasks.md`
- #5933: `specs/5933/spec.md`, `specs/5933/plan.md`, `specs/5933/tasks.md`
- #5934: `specs/5934/spec.md`, `specs/5934/plan.md`, `specs/5934/tasks.md`
- #5935: `specs/5935/spec.md`, `specs/5935/plan.md`, `specs/5935/tasks.md`
- #5936: `specs/5936/spec.md`, `specs/5936/plan.md`, `specs/5936/tasks.md`
- #5937: `specs/5937/spec.md`, `specs/5937/plan.md`, `specs/5937/tasks.md`
- #5938: `specs/5938/spec.md`, `specs/5938/plan.md`, `specs/5938/tasks.md`
- #5939: `specs/5939/spec.md`, `specs/5939/plan.md`, `specs/5939/tasks.md`
- #5940: `specs/5940/spec.md`, `specs/5940/plan.md`, `specs/5940/tasks.md`
- #5941: `specs/5941/spec.md`, `specs/5941/plan.md`, `specs/5941/tasks.md`
- #5947: `specs/5947/spec.md`, `specs/5947/plan.md`, `specs/5947/tasks.md`

## Implementation Progress
- #5934 delivered governance structural-coupling telemetry and policy validation in combined shell-surface trend gates, with full fast CI-tools regression passing on the updated schema.

## Exit Criteria
1. Every R65 issue has SPECIFY/PLAN/TASKS artifacts committed and linked.
2. Each child task is delivered via TDD evidence and mapped AC verification.
3. Critical findings are closed with merged task PRs and traceable regression coverage.
4. Security and runtime gates are enforced in CI for sustained non-regression.

## Progress Notes
- 2026-02-25: #5933 phase-1 extraction delivered by introducing
  `crates/kamn-runtime-guards` and migrating runtime guard contracts behind
  `kamn-core` compatibility shims (`anti_spam`, `fairness_policy`,
  `quota_policy`, `message_delivery_guards`, `retention_engine`, `watchdog`).
- 2026-02-25: `#5936` wires Service API message-send runtime through M0-M11 module contracts and persists deterministic `data_layer_runtime_evidence` markers for each created message.
- 2026-02-25: `#5938` expands parser/protocol fuzz surfaces to signature-profile and Kolme API codec targets with deterministic corpus replay metadata and property invariants.
- 2026-02-25: `#5941` adds required cargo-audit CI policy enforcement with waiver-schema validation and archived security reports.
- 2026-02-25: `#5939` adds bounded critical-path mutation/coverage gates in workspace pre-merge CI with deterministic report artifacts and fail-closed policies.
- 2026-02-25: `#5935` deduplicates high-impact parser/helper classes through canonical shared modules in `kamn-kolme` and `kamn-mcp-server`, with unicode escape conformance tests and fail-closed duplicate inventory contracts.
