# Spec: Issue #5868 - Live E2E Fail-Closed CI Execution

- Issue: #5868
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
Live E2E scenarios need fail-closed CI execution against running Kolme + KAMN components; synthetic pass behavior is insufficient.

## Scope
In scope:
- Enforce fail-closed live probe prerequisites.
- Provide local orchestration entrypoint for Kolme + KAMN runtime.
- Integrate baseline live scenario lane into CI.

Out of scope:
- Upstream Kolme source changes.

## Acceptance Criteria
- AC-1: Live lane fails when prerequisites are missing.
- AC-2: Baseline scenarios execute against live runtime in CI.
- AC-3: Scenario output includes real interaction evidence.
- AC-4: Contracts/docs reflect fail-closed policy.
