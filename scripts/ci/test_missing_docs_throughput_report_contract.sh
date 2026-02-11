#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/missing_docs_throughput_report_contract.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

REPORT_PATH="$TMP_DIR/missing-docs-throughput-report.json"

if [ ! -x "$SCRIPT" ]; then
  echo "expected throughput report contract script to be executable" >&2
  exit 1
fi

python3 "$SCRIPT" generate \
  --output-json "$REPORT_PATH"

python3 "$SCRIPT" check \
  --report-file "$REPORT_PATH"

if ! grep -Fq '"schema_version": "kamn.ci.kamn-core-missing-docs-throughput-report.v1"' "$REPORT_PATH"; then
  echo "expected throughput report schema version marker" >&2
  exit 1
fi

if ! grep -Fq '"target_modules_per_100_commits": 5' "$REPORT_PATH"; then
  echo "expected throughput report target marker" >&2
  exit 1
fi

BAD_REPORT_PATH="$TMP_DIR/missing-docs-throughput-report-bad.json"
cp "$REPORT_PATH" "$BAD_REPORT_PATH"
sed -i 's/"reason_key": "throughput_target_[^"]*"/"reason_key": "invalid_reason"/' "$BAD_REPORT_PATH"
if python3 "$SCRIPT" check --report-file "$BAD_REPORT_PATH" >"$TMP_DIR/bad.out" 2>"$TMP_DIR/bad.err"; then
  echo "expected policy checker failure for invalid reason key" >&2
  cat "$TMP_DIR/bad.out" >&2 || true
  cat "$TMP_DIR/bad.err" >&2 || true
  exit 1
fi

echo "missing-docs throughput report contract tests passed."
