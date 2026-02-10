# Rustdoc Publishing Workflow

This guide defines the bounded `kamn-core` rustdoc generation and publication
path used by local contributors and CI lanes.

## Command Surface

- Generate docs for `kamn-core` only (no dependency docs):
  - `cargo doc -p kamn-core --no-deps`
- Fail closed on rustdoc warnings:
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p kamn-core --no-deps`

## Local Preview

- Build docs:
  - `cargo doc -p kamn-core --no-deps`
- Open generated docs:
  - `xdg-open target/doc/kamn_core/index.html`

## CI-Friendly Publication Path

- Build docs with warning-fail posture:
  - `RUSTDOCFLAGS="-D warnings" cargo doc -p kamn-core --no-deps`
- Package deterministic artifact:
  - `tar -czf /tmp/kamn-core-rustdoc.tar.gz -C target doc`

The artifact source is always `target/doc` and can be uploaded by CI workflows
without adding hosted-doc platform dependencies.

## Contract Enforcement

- Missing-doc policy checker:
  - `bash scripts/ci/check_kamn_core_missing_docs_policy.sh`
- Regression tests for checker behavior:
  - `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`

## Cost Posture

- Restrict scope to `-p kamn-core --no-deps` to avoid workspace-wide rustdoc
  builds in fast-gate lanes.
- CI selector only routes this checker for missing-doc policy and docs-contract
  paths.
