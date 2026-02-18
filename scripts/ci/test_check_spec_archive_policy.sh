#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_spec_archive_policy.sh"

if [ ! -x "$CHECKER" ]; then
  echo "expected spec archive policy checker to be executable: $CHECKER" >&2
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

fixture_output="$(
  bash "$CHECKER" \
    --repo-root "$MUTATED_ROOT" \
    --output-json "$TMP_DIR/fixture-report.json"
)"
if ! printf '%s\n' "$fixture_output" | grep -q '^status=ok$'; then
  echo "expected synthetic archive fixture to satisfy archive policy checker" >&2
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

echo "spec archive policy checker tests passed."
