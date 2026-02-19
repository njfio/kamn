# Issue #3955 Spec

- Title: Subtask: implement managed key-source adapter abstraction and provenance marker emission
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Managed-external signer execution enforces provenance checks, but the managed key-source path is not represented as an explicit adapter boundary with deterministic provenance marker output consumed by signer profile/key-source evaluation.

## Acceptance Criteria
- AC-1: Managed key-source execution is routed through an explicit adapter abstraction for managed-external signing.
- AC-2: Adapter output includes deterministic provenance marker fields for managed key-source/profile selection and signer public key material.
- AC-3: Signer profile/key-source path consumes the adapter provenance marker and fails closed on parity mismatch.
- AC-4: Unit, Functional, Integration, and Regression coverage exists and passes for the managed key-source adapter/provenance flow.
- AC-5: `docs/ops/configuration.md` documents the managed key-source adapter provenance mapping.

## Scope
In scope:
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `docs/ops/configuration.md`
- `specs/3955/spec.md`
- `specs/3955/plan.md`
- `specs/3955/tasks.md`

Out of scope:
- New external/HSM provider integrations.
- CI workflow or shell-lane expansions.
- Rotation freshness enforcement (`#3956`).

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Unit | managed key-source adapter output | provenance marker fields are deterministic and complete |
| C-02 | AC-2/AC-3 | Functional | managed-external signer flow with matching marker/selection | payload generation succeeds and marker parity check passes |
| C-03 | AC-1/AC-4 | Integration | managed profile matrix execution path | managed adapter path is used with existing provenance checks intact |
| C-04 | AC-3/AC-4 | Regression | managed marker/selection mismatch injection | fail-closed deterministic mismatch reason code |
| C-05 | AC-5 | Functional | ops configuration docs markers | docs declare managed adapter provenance mapping |

## Test Mapping
- `cargo test -p kamn-node signer::managed_backend::tests::unit_managed_key_source_adapter_emits_deterministic_provenance_marker -- --exact --nocapture`
- `cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection -- --exact --nocapture`
- `cargo test -p kamn-node signer::tests::regression_managed_key_source_provenance_marker_profile_mismatch_fails_closed -- --exact --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_managed_key_source_adapter_provenance_mapping -- --exact`

## Success Metrics
- Managed-external signer path has explicit adapter ownership and deterministic provenance marker parity enforcement.
- Shell LOC delta remains `0`.
- Rust LOC increases only for signer logic/tests/docs-contract updates.
