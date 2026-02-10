# DID Core 1.1 Conformance Profile for `kamn:did` (Issue #81)

This profile maps the `kamn:did` method schema to DID Core 1.1 conformance requirements and captures explicit decisions, known gaps, and future test vectors.

## DID Core Requirement Mapping
| DID Core Element | Requirement Level | kamn:did Status | Notes |
|---|---|---|---|
| id | REQUIRED | covered | DID subject identifier is mandatory. |
| verificationMethod | REQUIRED | covered | At least one verification method is required. |
| authentication | REQUIRED | covered | Authentication relationship must be present. |
| assertionMethod | RECOMMENDED | covered | Included for signed claim validation surfaces. |
| capabilityInvocation | RECOMMENDED | covered | Required by profile for operator-bound actions. |
| service | OPTIONAL | partial | Service rules remain profile-constrained. |

## Conformance Decisions
- Decision: require verificationMethod and capabilityInvocation.
- Decision: reject unsupported DID method prefixes.
- Decision: treat empty capability relationship arrays as invalid.
- Decision: canonical service endpoint is `kamn://messaging/<method-specific-id>` with lowercase scheme/authority and a single normalized path segment.
- Decision: endpoint queries/fragments and multi-segment paths are non-conformant.
- Decision: unsupported or mixed verification method algorithms remain non-conformant for `kamn:did` baseline profile.

## Candidate Test Vectors
- Vector-C1: valid kamn:did document with required relationships.
- Vector-C2: valid update preserving DID subject continuity.
- Vector-C3: canonicalization normalizes uppercase scheme/authority/identifier to canonical endpoint form.
- Vector-N1: document missing id is rejected.
- Vector-N2: unsupported verification method algorithm is rejected.
- Vector-N3: service endpoint with unsupported scheme, query/fragment, or multi-segment path is rejected.
- Vector-N4: mixed verification method algorithm sets are rejected for baseline profile.
- Vector-M1: migration matrix allows approved multikey transitions and blocks downgrade/unsupported paths.
- Previously non-conformant document (missing id) must be rejected.

## Downstream Test Category Mapping
- Unit: schema-field validator mappings.
- Functional: DID document acceptance and rejection examples.
- Integration: DID registry interaction expectations.
- Regression: previously non-conformant examples remain rejected.

## Service Endpoint Canonicalization Conformance Contract
- Vector fixture:
  - `fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json`
- Matrix runner:
  - `python3 scripts/did/run_service_endpoint_canonicalization_matrix.py --fixture fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json --output-json /tmp/did-service-endpoint-canonicalization-matrix-report.json`
- Evidence bundle generator:
  - `bash scripts/did/generate_service_endpoint_canonicalization_evidence_bundle.sh --output-file /tmp/did-service-endpoint-canonicalization.json --fixture fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_service_endpoint_canonicalization_policy.sh --bundle-file /tmp/did-service-endpoint-canonicalization.json`
- PR fast contract lane:
  - `bash scripts/did/run_service_endpoint_canonicalization_contract_lane.sh --output-file /tmp/did-service-endpoint-canonicalization-contract.json`
- Required reason-key markers:
  - `did_service_endpoint_canonicalization_reason_codes:GO:v1`
  - `did_service_endpoint_canonicalization_reason_codes:NO-GO:v1`
- Required fail-closed policy:
  - non-canonical service endpoint scheme/authority/path combinations must remain rejected (`Regression: #1000`).

## Multi-Key Algorithm Mixing and Migration Matrix Contract
- Vector fixture:
  - `fixtures/did_core_conformance/multikey_algorithm_migration_vectors.json`
- Matrix runner:
  - `python3 scripts/did/run_multikey_algorithm_migration_matrix.py --fixture fixtures/did_core_conformance/multikey_algorithm_migration_vectors.json --output-json /tmp/did-multikey-algorithm-migration-matrix-report.json`
- Evidence bundle generator:
  - `bash scripts/did/generate_multikey_algorithm_policy_evidence_bundle.sh --output-file /tmp/did-multikey-algorithm-policy.json --fixture fixtures/did_core_conformance/multikey_algorithm_migration_vectors.json --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/did/check_multikey_algorithm_policy.sh --bundle-file /tmp/did-multikey-algorithm-policy.json`
- PR fast contract lane:
  - `bash scripts/did/run_multikey_algorithm_policy_contract_lane.sh --output-file /tmp/did-multikey-algorithm-policy-contract.json`
- Required reason-key markers:
  - `did_multikey_algorithm_policy_reason_codes:GO:v1`
  - `did_multikey_algorithm_policy_reason_codes:NO-GO:v1`
- Required fail-closed policy:
  - mixed or unsupported verification method algorithm sets must remain rejected under migration policy checks (`Regression: #1001`).

## Local Validation
Run from repository root:

```bash
bash scripts/did/test_generate_service_endpoint_canonicalization_evidence_bundle.sh
bash scripts/did/test_run_service_endpoint_canonicalization_matrix.sh
bash scripts/did/test_run_service_endpoint_canonicalization_contract_lane.sh
bash scripts/did/test_generate_multikey_algorithm_policy_evidence_bundle.sh
bash scripts/did/test_run_multikey_algorithm_migration_matrix.sh
bash scripts/did/test_run_multikey_algorithm_policy_contract_lane.sh
cargo test -p kamn-core --test did_core_conformance_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
