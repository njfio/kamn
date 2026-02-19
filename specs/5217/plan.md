# Issue #5217 Plan

- Issue: #5217
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Create a migration-regression test that expects a new wave2 matrix harness and retired singleton file inventory (RED expected initially).
2. Implement `docs_contract_matrix_wave2_harness.rs` with explicit case IDs, document labels, and required markers for each migrated singleton suite.
3. Delete superseded singleton docs test files included in this wave.
4. Update any lane wrappers/selector routing that referenced retired singleton test names, keeping shell LOC neutral.
5. Run targeted matrix + migration tests, then broader docs-contract regression checks.
6. Update issue process logs and set spec status to Implemented on completion.

## Targeted Wave
- `tls_feature_gate_ci_docs.rs`
- `persistence_live_validation_roadmap_docs.rs`
- `data_governance_retention_docs.rs`
- `key_management_and_encryption_docs.rs`
- `sdk_example_fixture_drift_docs.rs`
- `group_sender_key_rotation_docs.rs`
- `incident_readiness_docs.rs`
- `python_sdk_beta_docs.rs`
- `typescript_sdk_beta_docs.rs`
- `service_marketplace_docs.rs`
- `shell_surface_governance_docs.rs`
- `testing_structure_docs.rs`

## Risks and Mitigations
- Risk: Missing parity when moving assertions.
  - Mitigation: Keep one-to-one marker mapping with stable case IDs and migration regression inventory.
- Risk: Over-broad consolidation causing brittle tests.
  - Mitigation: Limit wave to explicit bounded file list; keep small focused cases.
- Risk: Hidden coupling to file names.
  - Mitigation: Add regression contract test to enforce expected retirements explicitly.

## Interfaces / Contracts
- New harness matrix struct:
  - `case_id`
  - `document_label`
  - `document`
  - `required_markers`
- Migration regression contract:
  - verifies wave2 harness exists
  - verifies retired singleton files are absent
- Wrapper compatibility:
  - lane wrappers invoke `docs_contract_matrix_wave2_harness` instead of deleted singleton binaries
