#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_spec_archive_policy.sh"
ARCHIVE_TOOL="$ROOT_DIR/scripts/ci/archive_completed_specs.py"

if [ ! -x "$CHECKER" ]; then
  echo "expected spec archive policy checker to be executable: $CHECKER" >&2
  exit 1
fi
if [ ! -x "$ARCHIVE_TOOL" ]; then
  echo "expected spec archive migration tool to be executable: $ARCHIVE_TOOL" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_REPORT="$TMP_DIR/pass-report.json"
pass_output="$(
  bash "$CHECKER" \
    --repo-root "$ROOT_DIR" \
    --output-json "$PASS_REPORT"
)"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected status=ok marker for valid spec archive state" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^final_decision=GO$'; then
  echo "expected final_decision=GO marker for valid spec archive state" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes=none$'; then
  echo "expected reason_codes=none marker for valid spec archive state" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.spec-archive-policy-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker for spec archive policy checker" >&2
  exit 1
fi

MUTATED_ROOT="$TMP_DIR/mutated-root"
mkdir -p "$MUTATED_ROOT/specs/archive/9999" "$MUTATED_ROOT/specs/9999"
cat > "$MUTATED_ROOT/specs/archive/9999/spec.md" <<'EOF'
# Spec — Issue #9999

- Status: Implemented
EOF
cat > "$MUTATED_ROOT/specs/archive/9999/plan.md" <<'EOF'
# Plan — Issue #9999
EOF
cat > "$MUTATED_ROOT/specs/archive/9999/tasks.md" <<'EOF'
# Tasks — Issue #9999
EOF
cat > "$MUTATED_ROOT/specs/9999/ARCHIVED.md" <<'EOF'
# Archived Spec Pointer

- issue_id: 9999
- archive_path: specs/archive/9999
EOF
cat > "$MUTATED_ROOT/specs/archive/index.md" <<'EOF'
# Archived Spec Index

- schema_version: kamn.specs.archive-index-report.v1
- archived_issue_count: 1

| issue_id | title | archived_on | archive_path | pointer_path |
|---|---|---|---|---|
| 9999 | Synthetic Archived Fixture | 2026-02-18 | `specs/archive/9999` | `specs/9999/ARCHIVED.md` |
EOF

fixture_output="$(
  bash "$CHECKER" \
    --repo-root "$MUTATED_ROOT" \
    --output-json "$TMP_DIR/fixture-report.json"
)"
if ! printf '%s\n' "$fixture_output" | grep -q '^status=ok$'; then
  echo "expected synthetic archive fixture to satisfy archive policy checker" >&2
  exit 1
fi

MUTATED_NO_INDEX="$TMP_DIR/mutated-no-index"
cp -R "$MUTATED_ROOT" "$MUTATED_NO_INDEX"
rm -f "$MUTATED_NO_INDEX/specs/archive/index.md"

set +e
missing_index_output="$(
  bash "$CHECKER" \
    --repo-root "$MUTATED_NO_INDEX" \
    --output-json "$TMP_DIR/missing-index-report.json" 2>&1
)"
missing_index_exit=$?
set -e

if [ "$missing_index_exit" -eq 0 ]; then
  echo "expected checker to fail when archive index report is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_index_output" | grep -q 'spec_archive_index_missing'; then
  echo "expected deterministic spec_archive_index_missing reason marker when archive index report is removed" >&2
  exit 1
fi

rm -f "$MUTATED_ROOT/specs/9999/ARCHIVED.md"

set +e
missing_pointer_output="$(
  bash "$CHECKER" \
    --repo-root "$MUTATED_ROOT" \
    --output-json "$TMP_DIR/missing-pointer-report.json" 2>&1
)"
missing_pointer_exit=$?
set -e

if [ "$missing_pointer_exit" -eq 0 ]; then
  echo "expected checker to fail when archived spec pointer is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_pointer_output" | grep -q 'spec_archive_pointer_missing'; then
  echo "expected deterministic spec_archive_pointer_missing reason marker when pointer is removed" >&2
  exit 1
fi

TOOL_ROOT="$TMP_DIR/tool-root"
mkdir -p "$TOOL_ROOT/specs/9000"
cat > "$TOOL_ROOT/specs/9000/spec.md" <<'EOF'
# Spec — Issue #9000

- Title: Synthetic Archive Tool Fixture
- Status: Implemented
EOF
cat > "$TOOL_ROOT/specs/9000/plan.md" <<'EOF'
# Plan — Issue #9000
EOF
cat > "$TOOL_ROOT/specs/9000/tasks.md" <<'EOF'
# Tasks — Issue #9000
EOF

tool_dry_run_output="$(
  python3 "$ARCHIVE_TOOL" \
    --repo-root "$TOOL_ROOT" \
    --issue-id 9000 \
    --output-json "$TMP_DIR/tool-dry-run-report.json"
)"
if ! printf '%s\n' "$tool_dry_run_output" | grep -q '^status=ok$'; then
  echo "expected archive tool dry-run status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tool_dry_run_output" | grep -q '^mode=dry-run$'; then
  echo "expected archive tool dry-run mode marker" >&2
  exit 1
fi
if [ ! -f "$TOOL_ROOT/specs/9000/spec.md" ]; then
  echo "dry-run should not move source spec files" >&2
  exit 1
fi
if [ -d "$TOOL_ROOT/specs/archive/9000" ]; then
  echo "dry-run should not create archive target directory" >&2
  exit 1
fi

tool_apply_output="$(
  python3 "$ARCHIVE_TOOL" \
    --repo-root "$TOOL_ROOT" \
    --issue-id 9000 \
    --apply \
    --archived-on 2026-02-18 \
    --output-json "$TMP_DIR/tool-apply-report.json"
)"
if ! printf '%s\n' "$tool_apply_output" | grep -q '^status=ok$'; then
  echo "expected archive tool apply status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tool_apply_output" | grep -q '^mode=apply$'; then
  echo "expected archive tool apply mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$tool_apply_output" | grep -q '^archived_issue_count=1$'; then
  echo "expected archive tool to archive exactly one issue in apply mode" >&2
  exit 1
fi
if [ ! -f "$TOOL_ROOT/specs/archive/9000/spec.md" ]; then
  echo "expected archive tool to move spec.md into archive target directory" >&2
  exit 1
fi
if [ ! -f "$TOOL_ROOT/specs/9000/ARCHIVED.md" ]; then
  echo "expected archive tool to write active-tree ARCHIVED.md pointer" >&2
  exit 1
fi
if ! grep -q 'archive_path: specs/archive/9000' "$TOOL_ROOT/specs/9000/ARCHIVED.md"; then
  echo "expected archive tool pointer to include archive path marker" >&2
  exit 1
fi
if [ ! -f "$TOOL_ROOT/specs/archive/index.md" ]; then
  echo "expected archive tool to generate archive index report" >&2
  exit 1
fi
if ! grep -q '| 9000 | Synthetic Archive Tool Fixture |' "$TOOL_ROOT/specs/archive/index.md"; then
  echo "expected archive index report to include archived issue mapping row" >&2
  exit 1
fi

tool_checker_output="$(
  bash "$CHECKER" \
    --repo-root "$TOOL_ROOT" \
    --output-json "$TMP_DIR/tool-checker-report.json"
)"
if ! printf '%s\n' "$tool_checker_output" | grep -q '^status=ok$'; then
  echo "expected checker to accept archive output produced by archive tool" >&2
  exit 1
fi
if ! printf '%s\n' "$tool_checker_output" | grep -q '^archived_issue_count=1$'; then
  echo "expected checker archived_issue_count marker to match tool output fixture" >&2
  exit 1
fi
if ! printf '%s\n' "$tool_checker_output" | grep -q '^index_entry_count=1$'; then
  echo "expected checker index_entry_count marker to match tool output fixture" >&2
  exit 1
fi

echo "spec archive policy checker tests passed."
