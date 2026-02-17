# Milestone Index: R27.29

Milestone: R27.29 Observability, transport resilience, and TLS governance convergence
GitHub Milestone: https://github.com/njfio/kamn/milestone/63
Status: In Progress

## Objective

Converge deterministic retry/reconnect transport contracts, API-runtime observability schema governance,
and TLS evidence completeness checks with fail-closed release-gate reason mapping.

## Scope

- Retry/backoff/reconnect envelope governance for live transport lanes.
- API-runtime-kolme observability schema convergence checks.
- TLS evidence completeness verification and deterministic release-go/no-go reason mapping.
- CI smoke vs local-heavy boundary enforcement for converged governance checks.

## Issue Hierarchy

- Epic: #4293
- Stories: #4294, #4295
- Tasks: #4296, #4297, #4298, #4299
- Subtasks: #4300, #4301, #4302, #4303, #4304, #4305, #4306, #4307

## Exit Signals

- Transport retry/reconnect drift fails closed with deterministic reason codes.
- API-runtime-kolme observability schema drift fails closed with stable reason mapping.
- TLS evidence completeness and freshness are validated before release promotion.
- Docs and docs-contract tests remain synchronized with reason taxonomy markers.
