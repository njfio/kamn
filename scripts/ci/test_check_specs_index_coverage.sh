#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CHECKER="$ROOT_DIR/scripts/ci/check_specs_index_coverage.sh"
INDEX_DOC="$ROOT_DIR/specs/INDEX.md"
CONTRIBUTING_FILE="$ROOT_DIR/.github/CONTRIBUTING.md"
CI_TOOLS_FILE="$ROOT_DIR/scripts/ci/test_ci_tools.sh"

if [ ! -x "$CHECKER" ]; then
  echo "expected specs index coverage checker to be executable: $CHECKER" >&2
  exit 1
fi

if [ ! -f "$INDEX_DOC" ]; then
  echo "expected specs index entrypoint document: $INDEX_DOC" >&2
  exit 1
fi

if ! grep -Fq 'bash scripts/ci/check_specs_index_coverage.sh --output-json /tmp/specs-index-coverage.json' "$CONTRIBUTING_FILE"; then
  echo "expected contributor guidance to mention specs index coverage verification command" >&2
  exit 1
fi

if ! grep -Fq 'bash "$ROOT_DIR/scripts/ci/test_check_specs_index_coverage.sh"' "$CI_TOOLS_FILE"; then
  echo "expected CI tools regression lane to run specs index coverage contract test" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

expect_reason() {
  local output="$1"
  local reason="$2"
  if ! printf '%s\n' "$output" | grep -q "$reason"; then
    echo "expected deterministic $reason reason marker" >&2
    exit 1
  fi
}

run_expect_failure() {
  local repo_root="$1"
  local report_path="$2"
  local reason="$3"
  local description="$4"
  local output=""

  set +e
  output="$(bash "$CHECKER" --repo-root "$repo_root" --output-json "$report_path" 2>&1)"
  local exit_code=$?
  set -e

  if [ "$exit_code" -eq 0 ]; then
    echo "expected specs index checker to fail when $description" >&2
    exit 1
  fi
  expect_reason "$output" "$reason"
}

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

run_expect_failure \
  "$MISSING_ENTRY_ROOT" \
  "$TMP_DIR/missing-entry-report.json" \
  "specs_index_missing_entry" \
  "a top-level spec is omitted from shards"

EXTRA_ENTRY_ROOT="$TMP_DIR/extra-entry-root"
cp -R "$PASS_ROOT/." "$EXTRA_ENTRY_ROOT"
cat > "$EXTRA_ENTRY_ROOT/specs/index/6500-6999.md" <<'EOF'
# Specs Index Shard 6500-6999

- [6501-beta.md](../6501-beta.md)
- [6999-missing.md](../6999-missing.md)
EOF

run_expect_failure \
  "$EXTRA_ENTRY_ROOT" \
  "$TMP_DIR/extra-entry-report.json" \
  "specs_index_unknown_entry" \
  "a shard references a missing top-level spec"

echo "specs index coverage checker tests passed."
