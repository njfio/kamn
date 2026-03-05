# Spec: Issue #6380 - Fail fast on TLS-disabled production runtime paths

## Objective

Enforce fail-closed Service API TLS policy on production runtime paths so startup aborts when TLS is disabled or TLS material is missing, while preserving explicit local-only opt-in behavior for loopback-bound workflows.

## Inputs/Outputs

- Inputs:
  - Service API TLS resolver and startup wiring in `crates/kamn-node/src/service_api_endpoint/server.rs`
  - Service API runtime snapshot (`runtime_mode`) and bind address at startup
  - Deployment guidance in `docs/ops/deployment.md`
- Outputs:
  - deterministic runtime-path TLS policy enforcement for production-targeted modes
  - contract tests covering production fail-closed and local loopback opt-in paths
  - deployment/local-override documentation for operators

## Boundaries/Non-goals

- In scope:
  - Runtime-path policy enforcement around existing Service API TLS mode resolution.
  - Deterministic diagnostics for policy violations.
  - Contract tests in Service API TLS resolver coverage.
  - Operator docs clarifying production requirement and local loopback exception.
- Out of scope:
  - Rewriting TLS implementation or crypto primitives.
  - Modifying CI workflows/scripts in this issue.
  - Changing observability endpoint TLS policy.

## Failure modes

- FM-1: Production-targeted runtime path accepts `KAMN_SERVICE_API_TLS_MODE=disabled` on non-loopback bind addresses.
- FM-2: Diagnostics do not clearly indicate TLS policy remediation when fail-closed is triggered.
- FM-3: Contract tests miss production runtime regressions for explicit disabled mode handling.
- FM-4: Local non-production guidance is ambiguous, causing unsafe rollout assumptions.

## Acceptance criteria (testable booleans)

- [ ] AC-1: Service API startup fails closed when TLS mode resolves to disabled on production-targeted runtime modes with non-loopback bind addresses.
- [ ] AC-2: Service API startup diagnostics include deterministic TLS policy violation markers/remediation for blocked production paths.
- [ ] AC-3: Existing local workflows remain explicitly opt-in via loopback-bound Service API startup and are documented.
- [ ] AC-4: Service API TLS contract tests cover production enforcement behavior (fail-closed + allowed local loopback path).

## Files to touch

- `specs/6380-fail-fast-tls-production-paths.md`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `docs/ops/deployment.md`

## Error semantics

- Fail closed with non-zero startup error when runtime-path policy blocks disabled TLS on production-targeted paths.
- Preserve existing fail-closed behavior for required TLS material validation.
- Emit deterministic error strings that include runtime mode, bind address class, and remediation markers.
- No silent fallback to insecure mode on non-loopback production-targeted paths.

## Test plan

- RED:
  - Add/adjust Service API TLS resolver tests asserting production-targeted runtime paths reject disabled mode on non-loopback bind addresses.
  - Add tests asserting loopback-bound local path remains explicitly allowed.
- GREEN:
  - Implement runtime-path TLS policy enforcement in resolver/startup wiring.
  - Ensure diagnostics include deterministic remediation guidance.
- REFACTOR:
  - Keep policy helpers focused (runtime-mode classification and bind-address classification).
- INTEGRATION:
  - Run targeted `kamn-node` Service API TLS tests covering startup behavior.
  - Run docs marker tests affected by deployment guidance updates.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
