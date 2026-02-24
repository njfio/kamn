# Plan: Issue #5866 - Service API Durable Persistence Continuity

- Issue: #5866
- Spec: `specs/5866/spec.md`
- Status: Draft
- Last Updated: 2026-02-24

## Approach
1. Audit existing Service API durable store read/write/load behavior across mutation families.
2. Add RED tests for uncovered restart continuity/no-op write behaviors.
3. Implement minimal store/save-path fixes.
4. Verify with scoped integration/regression/quality commands.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint/message_store.rs`
- `crates/kamn-node/src/service_api_endpoint/state_io.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/service_api_endpoint/tests.rs`

## Risks and Mitigations
- Risk: over-writing state on retrieval paths.
  - Mitigation: explicit save-on-mutation-only assertions.
- Risk: schema drift breaks reload.
  - Mitigation: fail-closed parse tests and integration restart checks.

## Interfaces / Contracts
- Durable state file schema remains `kamn.runtime.service-api-message-store.v2`.
- Mutation routes only: durable store writes permitted.
- Read-only routes: no mutation/no write side effects.

## ADR Requirement
- Not required.
