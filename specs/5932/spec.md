# Spec: Issue #5932 - Task: Networking transport hardening (pooling, health validation, multi-thread runtime, websocket keepalive)

- Issue: #5932
- Status: Implemented
- Type: task
- Priority: P1
- Area: networking
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5918

## Problem Statement
Current behavior includes single-thread runtime bottlenecks, no pooling, weak health validation, and weak websocket lifecycle handling.

## Scope
In scope:
- Adopt multi-thread runtime, connection pooling, strict health checks, and WS ping/persistent stream semantics.

Out of scope:
- Protocol redesign outside current transport contracts.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Service API runs on multi-threaded runtime with validated concurrency behavior.
- AC-2: Kolme transport reuses pooled connections and health probes validate HTTP status contracts.
- AC-3: WebSocket sessions support persistent streams with ping/pong and idle cleanup.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): Verify Service API runs on multi-threaded runtime with validated concurrency behavior.
- C-02 (Functional, AC-2): Verify Kolme transport reuses pooled connections and health probes validate HTTP status contracts.
- C-03 (Functional, AC-3): Verify WebSocket sessions support persistent streams with ping/pong and idle cleanup.
- C-04 (Functional, AC-4): Verify Unit, Functional, Integration, and Regression tests are present and passing.

## Success Metrics / Observable Signals
- AC-1 verification tests pass in scoped CI runs.
- AC-2 verification tests pass in scoped CI runs.
- AC-3 verification tests pass in scoped CI runs.
- AC-4 verification tests pass in scoped CI runs.

## Implementation Evidence (2026-02-25)
- AC-1: `serve_service_api_endpoint` and `serve_observability_endpoint` now build multi-thread Tokio runtimes with explicit worker-thread contracts.
- AC-2: `run_full_supervisor_http_probe` now fails closed on non-2xx HTTP status and `KolmeRuntimeCommitHttpTransport` now reuses keep-alive HTTP connections via bounded pooling.
- AC-3: Websocket persistent stream + ping/pong + idle cleanup behavior remained active and covered by existing websocket endpoint tests; no regression path introduced by this issue.
- AC-4: Added/updated unit and regression tests for probe classification, runtime threading contracts, and HTTP keep-alive pooling behavior.


## Required Test Categories
- Unit: transport config and health-check classifiers
- Functional: pooled HTTP and websocket lifecycle behavior
- Integration: concurrent load + ws stream persistence tests
- Regression: one-message-close and weak health probe behavior removed
- Performance: throughput/latency non-regression under concurrent load

## Dependencies
- #5918
