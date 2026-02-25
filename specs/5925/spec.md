# Spec: Issue #5925 - Task: Replace FNV-based key lifecycle audit chain hashing with cryptographic hash

- Issue: #5925
- Status: Implemented
- Type: task
- Priority: P1
- Area: security
- Milestone: `specs/milestones/r65-security-runtime-remediation-and-production-readiness/index.md`
- Last Updated: 2026-02-25
- Parent: Parent story: #5916

## Problem Statement
FNV-based hashing is collision-prone and weak for audit integrity claims.

## Scope
In scope:
- Replace audit-chain hash with SHA-256/BLAKE3 and version marker migration.

Out of scope:
- Audit UI/reporting feature expansion unrelated to hash integrity.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: Audit chain records use cryptographic hash with explicit version marker.
- AC-2: Collision-oriented adversarial tests fail to violate chain integrity assumptions.
- AC-3: Backward-compatibility migration path is documented and tested.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases
- C-01 (Functional, AC-1): `spec_c01_issue_5925_audit_records_use_versioned_sha256_marker` verifies `KeyLifecycle::audit_records()` emits `sha256:v1:<64-hex>` record hashes.
- C-02 (Functional, AC-2): `spec_c02_issue_5925_collision_style_payload_mutation_changes_hash` verifies adversarial payload mutation changes audit hash output.
- C-03 (Functional, AC-3): `spec_c03_issue_5925_legacy_v0_records_verify_for_migration` verifies legacy v0 records still verify through migration path; `docs/foundation/key-lifecycle-audit-trails.md` documents compatibility behavior.
- C-04 (Functional, AC-4): `cargo test -p kamn-core --test key_lifecycle -- --nocapture`, `cargo test -p kamn-core --test docs_contract_wave4_harness key_lifecycle_audit_trails_docs -- --nocapture`, `cargo fmt --check`, and `cargo clippy -p kamn-core -- -D warnings` pass.

## Success Metrics / Observable Signals
- Generated key lifecycle audit records carry explicit cryptographic version marker `sha256:v1:`.
- Legacy v0 records remain verifiable for migration without weakening v1 emission defaults.
- Collision-style payload mutation produces distinct hashes and fails verification when tampered.
- Scoped fmt/clippy/test/mutation gates pass for touched modules.


## Required Test Categories
- Unit: hash-chain computation and versioning
- Functional: audit chain append/verify
- Integration: key lifecycle audit persistence and retrieval
- Regression: FNV path removed from production audit integrity checks
- Performance: chain update overhead non-regression

## Dependencies
- #5916
