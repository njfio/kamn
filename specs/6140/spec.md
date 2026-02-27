# Spec: Issue #6140 - Task: [X-02] Decompose oversized route dispatcher

- Issue: #6140
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `r68-r59-swarm-remediation-and-full-gap-closure`
- Last Updated: 2026-02-27
- Parent: #6101

## Problem Statement
`handle_service_api_http_route` in `crates/kamn-node/src/service_api_endpoint/middleware_impl.rs` was a dense monolithic route dispatcher, increasing audit and maintenance risk called out in R59 finding `X-02`.

## Scope
In scope:
- Decompose HTTP dispatch flow into method-level helper functions while preserving route behavior.
- Add structural conformance coverage that enforces helper delegation.
- Keep route outcomes, status codes, and reason codes unchanged.

Out of scope:
- Semantic behavior changes to service API route contracts.
- New API endpoints or schema changes.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `handle_service_api_http_route` delegates to focused helper dispatch functions rather than containing full monolithic route logic.
- AC-2: Existing route behavior remains unchanged for POST and GET route families.
- AC-3: Regression/conformance tests fail closed if dispatcher monolith structure regresses.

## Conformance Cases
- C-01 (AC-1, Unit/Conformance): Structural test validates top-level handler delegates to method-level helpers.
- C-02 (AC-2, Functional/Regression): Existing service API route behavior tests continue passing for representative POST/GET routes.
- C-03 (AC-3, Regression/Conformance): Regression test fails if helper delegation markers disappear.

## Success Metrics / Observable Signals
- Targeted R59 finding `X-02` is remediated by merged code and test evidence.
- Scoped verification commands pass in local and CI runs.
- Closure evidence links PR and AC-to-test mapping.
