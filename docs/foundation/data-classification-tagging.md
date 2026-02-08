# Data Classification and Write-Tagging Slice (Issues #156, #157)

This document describes the first implementation slice for classification tiers and write-path tagging enforcement.

## Scope Delivered
- Added `DataClassificationEngine` to enforce write-path classification controls.
- Added versionable classification model:
  - `Public`
  - `Internal`
  - `Sensitive`
  - `Restricted`
- Added domain-level minimum classification policy:
  - `messages`, `tasks`, `escrows`, `reputation`
- Added required-tag enforcement by classification level.
- Enforced typed write-path failures for:
  - Missing required tags
  - Classification below domain minimum
  - Sensitive/restricted writes without tags
  - Invalid actor DIDs and malformed policy/tag definitions
- Added operator-facing status surface (`ClassificationStatus`) for deterministic control visibility.
- Integrated canonical write-key output through existing state key normalization.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test data_classification_tagging
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add classification inheritance for nested write contexts.
- Add policy-version tracking and migration rules for classification updates.
