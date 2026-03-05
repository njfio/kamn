# Spec: Issue #6381 - Document service secret rotation and key lifecycle operations

## Objective

Add a versioned operator runbook that defines deterministic service-secret rotation steps (generation, staged rollout, rollback, verification) for Service API auth keys and related runtime signer secrets.

## Inputs/Outputs

- Inputs:
  - existing Service API auth env contracts in `crates/kamn-core/src/signature_profile.rs`
  - runtime signer secret policy contracts in `docs/ops/configuration.md`
  - contributor and deployment docs (`.github/CONTRIBUTING.md`, `docs/ops/deployment.md`)
- Outputs:
  - versioned service-secret rotation runbook under `docs/ops/runbooks/`
  - explicit env-var ownership matrix and staged rotation procedure
  - validation checklist mapped to existing runtime/CI contract tests
  - cross-links from contributor and ops docs to the runbook

## Boundaries/Non-goals

- In scope:
  - documentation-only rotation runbook and documentation links.
  - deterministic validation checklist referencing existing tests/lanes.
  - docs contract test coverage for required runbook markers.
- Out of scope:
  - runtime cryptography or key derivation code changes.
  - secrets backend/provider migration.
  - CI workflow logic changes.

## Failure modes

- FM-1: runbook omits staged rollout/rollback sequence and operators rotate secrets unsafely.
- FM-2: required env variables and ownership boundaries are ambiguous.
- FM-3: runbook lacks validation steps tied to existing contract tests.
- FM-4: runbook exists but is not discoverable from contributor/ops docs.

## Acceptance criteria (testable booleans)

- [ ] AC-1: A versioned runbook documents key generation, staged rollout, rollback, and verification for service auth/runtime signer secrets.
- [ ] AC-2: Runbook documents required environment variables and explicit ownership boundaries.
- [ ] AC-3: Runbook includes a validation checklist mapped to existing runtime/CI contract tests.
- [ ] AC-4: Contributor and ops docs link to the runbook.
- [ ] AC-5: Docs contract tests enforce required runbook markers.

## Files to touch

- `specs/6381-document-service-secret-rotation-runbook.md`
- `docs/ops/runbooks/service-secret-rotation.md` (new)
- `docs/ops/deployment.md`
- `.github/CONTRIBUTING.md`
- `crates/kamn-core/tests/service_secret_rotation_runbook_docs.rs` (new)

## Error semantics

- N/A for runtime behavior (docs-only issue).
- Docs contract tests must fail closed if required runbook markers or cross-links drift.

## Test plan

- RED:
  - add docs contract test asserting runbook schema marker, required sections, env ownership markers, and link markers.
  - run the new docs test and observe failure before runbook/docs updates.
- GREEN:
  - add runbook + contributor/ops links and required markers.
  - re-run docs test to verify pass.
- REFACTOR:
  - keep runbook/test marker names concise and deterministic.
- INTEGRATION:
  - verify runbook is linked from contributor and ops docs.
  - execute docs contract test from `kamn-core` test suite.

## Phase 6 integration evidence

- Pending.

## Deviations

- None.
