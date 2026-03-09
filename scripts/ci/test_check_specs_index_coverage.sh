#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_specs_index_coverage.sh"
INDEX_DOC="$ROOT_DIR/specs/INDEX.md"

if [ ! -x "$CHECKER" ]; then
  echo "expected specs index coverage checker to be executable: $CHECKER" >&2
  exit 1
fi

if [ ! -f "$INDEX_DOC" ]; then
  echo "expected specs index entrypoint document: $INDEX_DOC" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

PASS_ROOT="$TMP_DIR/pass-root"
mkdir -p "$PASS_ROOT/specs/index"
cat > "$PASS_ROOT/specs/INDEX.md" <<'EOF'
# Specs Index

specs_index_version=kamn.docs.specs-index.v2
specs_index_scope=top_level_issue_specs_only
specs_index_naming_pattern=specs/{issue}-{slug}.md
specs_index_status_taxonomy_csv=planned,active,completed,superseded
specs_index_coverage_authority=scripts/ci/check_specs_index_coverage.sh
specs_index_shards_csv=specs/index/6000-6499.md,specs/index/6500-6999.md
EOF

cat > "$PASS_ROOT/specs/6001-alpha.md" <<'EOF'
# Spec 6001
EOF

cat > "$PASS_ROOT/specs/6501-beta.md" <<'EOF'
# Spec 6501
EOF

cat > "$PASS_ROOT/specs/index/6000-6499.md" <<'EOF'
# Specs Index Shard 6000-6499

- [6001-alpha.md](../6001-alpha.md)
EOF

cat > "$PASS_ROOT/specs/index/6500-6999.md" <<'EOF'
# Specs Index Shard 6500-6999

- [6501-beta.md](../6501-beta.md)
EOF

PASS_REPORT="$TMP_DIR/pass-report.json"
pass_output="$(bash "$CHECKER" --repo-root "$PASS_ROOT" --output-json "$PASS_REPORT")"

if ! printf '%s\n' "$pass_output" | grep -q '^status=ok$'; then
  echo "expected specs index checker success status on compliant fixture" >&2
  exit 1
fi

MISSING_ENTRY_ROOT="$TMP_DIR/missing-entry-root"
cp -R "$PASS_ROOT/." "$MISSING_ENTRY_ROOT"
cat > "$MISSING_ENTRY_ROOT/specs/index/6500-6999.md" <<'EOF'
# Specs Index Shard 6500-6999

EOF

set +e
missing_entry_output="$(bash "$CHECKER" --repo-root "$MISSING_ENTRY_ROOT" --output-json "$TMP_DIR/missing-entry-report.json" 2>&1)"
missing_entry_exit=$?
set -e

if [ "$missing_entry_exit" -eq 0 ]; then
  echo "expected specs index checker to fail when a top-level spec is omitted from shards" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_entry_output" | grep -q 'specs_index_missing_entry'; then
  echo "expected deterministic specs_index_missing_entry reason marker" >&2
  exit 1
fi

EXTRA_ENTRY_ROOT="$TMP_DIR/extra-entry-root"
cp -R "$PASS_ROOT/." "$EXTRA_ENTRY_ROOT"
cat > "$EXTRA_ENTRY_ROOT/specs/index/6500-6999.md" <<'EOF'
# Specs Index Shard 6500-6999

- [6501-beta.md](../6501-beta.md)
- [6999-missing.md](../6999-missing.md)
EOF

set +e
extra_entry_output="$(bash "$CHECKER" --repo-root "$EXTRA_ENTRY_ROOT" --output-json "$TMP_DIR/extra-entry-report.json" 2>&1)"
extra_entry_exit=$?
set -e

if [ "$extra_entry_exit" -eq 0 ]; then
  echo "expected specs index checker to fail when a shard references a missing top-level spec" >&2
  exit 1
fi
if ! printf '%s\n' "$extra_entry_output" | grep -q 'specs_index_unknown_entry'; then
  echo "expected deterministic specs_index_unknown_entry reason marker" >&2
  exit 1
fi

echo "specs index coverage checker tests passed."
