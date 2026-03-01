# Spec: Issue #6287 - Data-layer validated tagged hash contract

## Objective

Add fail-closed validation for tagged SHA-256 algorithm labels in `kamn-data-layer` and create
crate-level integration coverage for the public hashing API.

## Inputs/Outputs

- Inputs:
  - `value: &str`
  - `algorithm_label: &str`
- Outputs:
  - Success: `Ok("<label>:<64-char lowercase sha256 hex>")`
  - Failure: typed `DataLayerHashingError` for invalid labels

## Boundaries/Non-goals

- In scope:
  - New validated tagged-hash API in `crates/kamn-data-layer/src/data_layer_hashing.rs`
  - Typed error for label validation failures
  - Integration tests in `crates/kamn-data-layer/tests/`
- Out of scope:
  - New hash algorithms
  - Cross-crate wiring changes
  - New dependencies

## Failure Modes

- FM-1: empty algorithm labels are accepted.
- FM-2: malformed algorithm labels (non-lowercase-alnum-hyphen) are accepted.
- FM-3: valid labels produce unstable formatting or digest shape.

## Acceptance Criteria

- AC-1: `validated_tagged_sha256(value, algorithm_label)` exists and returns typed errors.
- AC-2: empty label returns `DataLayerHashingError::EmptyAlgorithmLabel`.
- AC-3: malformed label returns `DataLayerHashingError::InvalidAlgorithmLabel`.
- AC-4: valid canonical label returns deterministic `<label>:<digest>` output.
- AC-5: integration tests under `crates/kamn-data-layer/tests/` cover success and both error
  paths.
- AC-6: existing `kamn-data-layer` tests remain green.

## Files To Touch

- `crates/kamn-data-layer/src/data_layer_hashing.rs`
- `crates/kamn-data-layer/src/lib.rs` (exports if required)
- `crates/kamn-data-layer/tests/data_layer_hashing_integration.rs`
- `specs/6287-data-layer-validated-tagged-hash.md`

## Error Semantics

- Use typed enum errors only:
  - `EmptyAlgorithmLabel`
  - `InvalidAlgorithmLabel`
- No silent normalization or fallback.
- Existing `tagged_sha256` remains for compatibility; validated path is explicit.

## Test Plan

- RED:
  - Add integration tests referencing new validated API and typed errors.
  - Confirm tests fail before implementation.
- GREEN:
  - Implement minimal error type + validation + validated API.
- REFACTOR:
  - Extract small label validator helper.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-data-layer --tests -- -D warnings`
  - `cargo test -p kamn-data-layer`
