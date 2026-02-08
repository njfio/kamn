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

## Conformance Decisions and Open Questions
- Decision: require verificationMethod and capabilityInvocation.
- Decision: reject unsupported DID method prefixes.
- Decision: treat empty capability relationship arrays as invalid.
- Open question: service endpoint canonicalization strategy.
- Open question: multi-key algorithm mixing policy for future revisions.

## Candidate Test Vectors
- Vector-C1: valid kamn:did document with required relationships.
- Vector-C2: valid update preserving DID subject continuity.
- Vector-N1: document missing id is rejected.
- Vector-N2: unsupported verification method algorithm is rejected.
- Previously non-conformant document (missing id) must be rejected.

## Downstream Test Category Mapping
- Unit: schema-field validator mappings.
- Functional: DID document acceptance and rejection examples.
- Integration: DID registry interaction expectations.
- Regression: previously non-conformant examples remain rejected.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test did_core_conformance_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
