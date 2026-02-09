#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/check_kamn_core_missing_docs_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

CORE_LIB_FIXTURE="$TMP_DIR/lib.rs"
ALLOWLIST_FIXTURE="$TMP_DIR/allowlist.txt"
README_FIXTURE="$TMP_DIR/README.md"
PLAN_DOC_FIXTURE="$TMP_DIR/engineering-hardening-wave.md"

run_checker() {
  KAMN_CORE_LIB_PATH="$CORE_LIB_FIXTURE" \
  KAMN_CORE_MISSING_DOCS_ALLOWLIST_PATH="$ALLOWLIST_FIXTURE" \
  KAMN_README_PATH="$README_FIXTURE" \
  KAMN_ENGINEERING_HARDENING_DOC_PATH="$PLAN_DOC_FIXTURE" \
    bash "$SCRIPT"
}

reset_fixtures() {
  cp "$ROOT_DIR/crates/kamn-core/src/lib.rs" "$CORE_LIB_FIXTURE"
  cp "$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_allowlist.txt" "$ALLOWLIST_FIXTURE"
  cp "$ROOT_DIR/README.md" "$README_FIXTURE"
  cp "$ROOT_DIR/docs/planning/engineering-hardening-wave.md" "$PLAN_DOC_FIXTURE"
}

expect_failure() {
  local label="$1"
  if run_checker >"$TMP_DIR/checker.out" 2>"$TMP_DIR/checker.err"; then
    echo "$label: expected failure but checker succeeded." >&2
    cat "$TMP_DIR/checker.out" >&2 || true
    cat "$TMP_DIR/checker.err" >&2 || true
    exit 1
  fi
}

reset_fixtures
run_checker >/dev/null

# Regression: #896
reset_fixtures
sed -i '/#!\[warn(missing_docs)\]/d' "$CORE_LIB_FIXTURE"
expect_failure "missing warn policy should fail"

reset_fixtures
printf '\nsynthetic_module\n' >>"$ALLOWLIST_FIXTURE"
expect_failure "allowlist drift should fail"

reset_fixtures
sed -i '/check_kamn_core_missing_docs_policy.sh/d' "$README_FIXTURE"
expect_failure "README drift should fail"

reset_fixtures
sed -i '/#!\[warn(missing_docs)\]/d' "$PLAN_DOC_FIXTURE"
expect_failure "plan doc marker drift should fail"

echo "kamn-core missing-docs policy checker tests passed."
