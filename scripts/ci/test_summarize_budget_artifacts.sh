#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/ci/summarize_budget_artifacts.sh"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/a.json" <<'JSON'
{
  "lane": "fast-gate",
  "status": "pass",
  "elapsed_seconds": 100,
  "runner_minutes": 2,
  "changed_files": 2,
  "test_scope": "targeted",
  "cache_hit": "true",
  "retry_used": "false"
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/b.json" <<'JSON'
{
  "lane": "fast-gate",
  "status": "warn",
  "elapsed_seconds": 700,
  "runner_minutes": 12,
  "changed_files": 3,
  "test_scope": "full",
  "cache_hit": "false",
  "retry_used": "true"
}
JSON

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$TMP_DIR/c.json" <<'JSON'
{
  "lane": "deep-validate",
  "status": "pass",
  "elapsed_seconds": 2000,
  "runner_minutes": 35,
  "changed_files": 12,
  "test_scope": "full",
  "cache_hit": "unknown",
  "retry_used": "unknown"
}
JSON

out_fast="$TMP_DIR/out_fast.txt"
"$SCRIPT" --lane fast-gate "$TMP_DIR/a.json" "$TMP_DIR/b.json" "$TMP_DIR/c.json" > "$out_fast"

grep -q 'Records: 2' "$out_fast"
grep -q 'Lane filter: fast-gate' "$out_fast"
grep -q 'pass: 1' "$out_fast"
grep -q 'warn: 1' "$out_fast"
grep -q 'fail: 0' "$out_fast"
grep -q 'true: 1' "$out_fast"
grep -q 'false: 1' "$out_fast"
grep -q 'Narrow-diff records (<=3 changed files): 2' "$out_fast"
grep -q 'Narrow-diff elapsed mean: 400.00' "$out_fast"
grep -q 'Narrow-diff runner mean: 7.00' "$out_fast"
grep -q 'Narrow-diff full-scope count: 1' "$out_fast"

out_all="$TMP_DIR/out_all.txt"
"$SCRIPT" "$TMP_DIR/a.json" "$TMP_DIR/b.json" "$TMP_DIR/c.json" > "$out_all"
grep -q 'Records: 3' "$out_all"
grep -q 'Lane filter: all' "$out_all"

echo "summarize_budget_artifacts tests passed."
