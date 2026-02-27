# Issue 6221 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6185

## Problem Statement
`E2E Live Tests` on `main` fail because workflow-launched `kamn-node` processes do not set `KAMN_SERVICE_API_TLS_MODE`. The service API now defaults to fail-closed `require` mode when unset, which requires cert/key env vars and prevents node startup in the live CI lanes.

## Scope
In scope:
- Set explicit TLS mode env markers in `.github/workflows/e2e-live.yml` for live lanes that start external `kamn-node` processes.
- Add/extend workflow contract tests to enforce presence of TLS mode markers.

Out of scope:
- Changing service API TLS default behavior.
- Provisioning TLS certificates/keys in live CI for this issue.

## Acceptance Criteria
- AC-1: `e2e-live` workflow sets `KAMN_SERVICE_API_TLS_MODE=disable` in each live lane (`sdk-direct`, `mcp-agent`, `cli-smoke`) before starting `kamn-node`.
- AC-2: Workflow contract tests fail if those explicit TLS mode markers are removed.
- AC-3: `cargo test -p kamn-e2e-harness --test phase4i_ci_workflow_contract` passes with updated markers.

## Conformance Cases
- C-01 (AC-1, Conformance): `.github/workflows/e2e-live.yml` contains three explicit `KAMN_SERVICE_API_TLS_MODE=disable` markers in live run scripts.
- C-02 (AC-2, Unit/Conformance): `phase4i_ci_workflow_contract` asserts TLS mode marker presence/count for all three lanes.
- C-03 (AC-3, Functional): targeted contract test command passes.

## Success Metrics
- CI `E2E Live Tests` progresses past node startup and no longer fails due to missing `KAMN_SERVICE_API_TLS_CERT_FILE`.
