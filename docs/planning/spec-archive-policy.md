# Spec Archive Policy

This document defines the governance contract for archiving completed issue specs.

## Policy Markers

- `spec_archive_layout_version=kamn.spec-archive-layout.v1`
- `spec_archive_root=specs/archive`
- `spec_archive_index_path=specs/archive/index.md`
- `spec_archive_pointer_template=specs/<issue-id>/ARCHIVED.md`
- `spec_archive_required_artifacts=spec.md,plan.md,tasks.md`
- `spec_archive_retention_status_gate=Implemented`
- `spec_archive_retention_exceptions=audit|required-by-compliance`
- `spec_archive_policy_status=verified|fail-closed`

## Layout Contract

1. Archived issue specs live under `specs/archive/<issue-id>/`.
2. Each archived issue directory must contain `spec.md`, `plan.md`, and `tasks.md`.
3. Active tree location `specs/<issue-id>/` must retain an `ARCHIVED.md` pointer file with archive path metadata.
4. `specs/archive/index.md` must contain one mapping row per archived issue id and remain count-synchronized.

## Retention Contract

1. Only specs with status `Implemented` are eligible for archival.
2. Compliance/audit-required issue specs may remain active only when an explicit exception is documented.
3. Archive policy checks are fail-closed in CI through `scripts/ci/check_spec_archive_policy.sh`.
