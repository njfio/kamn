# Rustdoc Publishing Workflow

This guide defines the bounded `kamn-core` rustdoc generation and publication
path used by local contributors and CI lanes.

## Related Architecture Entrypoints

- Ownership boundaries:
  - `docs/architecture/kamn-core-module-map.md#ownership-matrix`
- Contributor command/reference map:
  - `docs/architecture/kamn-core-module-map.md#contributor-entrypoint-matrix`
- Hardening command baseline:
  - `docs/planning/engineering-hardening-wave.md#commands`

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
- Graduated-module guard fixture consumed by checker:
  - `fixtures/ci/kamn_core_missing_docs_graduated_modules.txt`
- Current graduated modules:
  - `bootstrap`, `key_recovery`, `kolme_runtime_commit`, `migrations`,
    `namespaces`, `smoke`, `state`, `task_lifecycle`
- Regression tests for checker behavior:
  - `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- Rustdoc artifact contract lane (bounded):
  - `bash scripts/ci/run_kamn_core_rustdoc_artifact_contract_lane.sh --output-json /tmp/kamn-core-rustdoc-artifact-report.json`
- Rustdoc artifact metadata/path policy checker:
  - `bash scripts/ci/check_kamn_core_rustdoc_artifact_policy.sh --report-file /tmp/kamn-core-rustdoc-artifact-report.json`

## Rustdoc Artifact Report Contract

- `schema_version`: `kamn.ci.kamn-core-rustdoc-artifact-report.v1`
- `crate`: must be `kamn-core`
- `command`: must be `RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps`
- `artifact_path`: tarball path
- `artifact_bytes`: positive integer
- `artifact_sha256`: lowercase 64-char hex digest
- `runtime_seconds` and `max_runtime_seconds`: bounded runtime fields
- `reason_key`: fail-closed policy reason marker

## Cost Posture

- Restrict scope to `-p kamn-core --no-deps` to avoid workspace-wide rustdoc
  builds in fast-gate lanes.
- CI selector only routes this checker for missing-doc policy and docs-contract
  paths.
- Graduated modules listed in `kamn_core_missing_docs_graduated_modules.txt`
  cannot be re-added to `#[allow(missing_docs)]` without a fail-closed policy
  break.
- Graduation update marker:
  - `Regression: #1828`
