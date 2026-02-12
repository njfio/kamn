#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REPORT_DOC_PATH="$ROOT_DIR/docs/planning/issues/missing-docs-first-batch-graduation-report.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

validate_report_doc() {
  local report_path="$1"

  if [ ! -f "$report_path" ]; then
    echo "expected missing-docs graduation batch report doc: $report_path" >&2
    return 1
  fi

  required_markers=(
    "Missing-Docs First Batch Graduation Report"
    "schema_version: kamn.ci.kamn-core-missing-docs-graduation-batch-report.v1"
    "batch_id: first-three-modules-v1"
    "bootstrap"
    "key_recovery"
    "kolme_runtime_commit"
    "fixtures/ci/kamn_core_missing_docs_graduated_modules.txt"
    "scripts/ci/check_kamn_core_missing_docs_policy.sh"
    "Regression: #2126"
  )

  local marker
  for marker in "${required_markers[@]}"; do
    if ! grep -Fq "$marker" "$report_path"; then
      echo "missing-docs graduation batch report contract failed: missing marker '$marker'" >&2
      return 1
    fi
  done
}

validate_report_doc "$REPORT_DOC_PATH"

MUTATED_REPORT_DOC="$TMP_DIR/mutated-report.md"
cp "$REPORT_DOC_PATH" "$MUTATED_REPORT_DOC"
sed -i '/schema_version: kamn.ci.kamn-core-missing-docs-graduation-batch-report.v1/d' \
  "$MUTATED_REPORT_DOC"

if validate_report_doc "$MUTATED_REPORT_DOC" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected graduation batch report validation to fail on schema marker drift" >&2
  exit 1
fi

grep -q "missing marker 'schema_version: kamn.ci.kamn-core-missing-docs-graduation-batch-report.v1'" \
  "$TMP_DIR/fail.out"

echo "missing-docs graduation batch report contract tests passed."
