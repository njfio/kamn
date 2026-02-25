# Spec: Issue #6003 - Add fail-closed full-supervisor lane liveness monitoring

- Issue: #6003
- Status: Reviewed
- Type: story
- Priority: P0
- Area: infra
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: #5917

## Problem Statement
In full runtime mode, service-api and observability lanes are started before daemon execution, but their liveness is only validated at final join. A lane can panic/exit during daemon execution and remain undetected until shutdown, violating fail-closed supervision expectations.

## Scope
In scope:
- Add active lane liveness monitoring while full-mode daemon execution is in progress.
- Fail closed immediately when a monitored full-supervisor lane exits before daemon completion.
- Emit deterministic lane-specific reason codes for service-api and observability lane liveness failures.
- Preserve successful behavior for normal full-mode execution paths.

Out of scope:
- New lane types.
- Runtime behavior changes for non-full modes.

## Risk Level
`high`

## Acceptance Criteria
- AC-1: Full-mode runtime monitors service-api and observability lane liveness during daemon execution (not only at final lane join).
- AC-2: Unexpected service-api lane exit during daemon execution fails closed with deterministic lane-specific reason code.
- AC-3: Unexpected observability lane exit during daemon execution fails closed with deterministic lane-specific reason code.
- AC-4: Existing normal full-mode execution paths remain green.

## Conformance Cases
- C-01 (Unit, AC-1): lane liveness guard reports healthy when monitored lane threads remain active while daemon runs.
- C-02 (Functional, AC-2): service-api lane early exit during daemon execution fails closed with `full_supervisor_service_api_lane_liveness_failed` reason code.
- C-03 (Functional, AC-3): observability lane early exit during daemon execution fails closed with `full_supervisor_observability_lane_liveness_failed` reason code.
- C-04 (Integration, AC-4): full-mode bootstrap/stop integration path still succeeds under nominal lane settings.

## Success Metrics / Observable Signals
- Full-mode runtime reports lane liveness failures immediately instead of only at shutdown join.
- Failure messages are deterministic and lane-specific.
- Existing full-mode integration contracts remain green.
