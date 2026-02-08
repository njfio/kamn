#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 [--lane <fast-gate|deep-validate>] <ci-budget-json> [more json files...]
USAGE
}

LANE_FILTER=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --lane)
      LANE_FILTER="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    --)
      shift
      break
      ;;
    *)
      break
      ;;
  esac
done

if [ "$#" -lt 1 ]; then
  usage >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for summarize_budget_artifacts.sh" >&2
  exit 2
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
VALUES_FILE="$TMP_DIR/values.tsv"

for file in "$@"; do
  if [ ! -f "$file" ]; then
    continue
  fi

  lane="$(jq -r '.lane // "unknown"' "$file")"
  if [ -n "$LANE_FILTER" ] && [ "$lane" != "$LANE_FILTER" ]; then
    continue
  fi

  elapsed="$(jq -r '.elapsed_seconds // 0' "$file")"
  runner="$(jq -r '.runner_minutes // 0' "$file")"
  status="$(jq -r '.status // "unknown"' "$file")"
  cache_hit="$(jq -r '.cache_hit // "unknown"' "$file")"
  retry_used="$(jq -r '.retry_used // "unknown"' "$file")"
  changed_files="$(jq -r '.changed_files // 0' "$file")"
  test_scope="$(jq -r '.test_scope // "unknown"' "$file")"

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$lane" \
    "$elapsed" \
    "$runner" \
    "$status" \
    "$cache_hit" \
    "$retry_used" \
    "$changed_files" \
    "$test_scope" \
    >> "$VALUES_FILE"
done

if [ ! -f "$VALUES_FILE" ] || [ ! -s "$VALUES_FILE" ]; then
  echo "No matching telemetry records found."
  exit 0
fi

count="$(wc -l < "$VALUES_FILE" | tr -d ' ')"

pctl() {
  local column="$1"
  local p="$2"
  local sorted="$TMP_DIR/sorted_${column}.txt"
  awk -F '\t' -v c="$column" '{print $c}' "$VALUES_FILE" | sort -n > "$sorted"
  local n
  n="$(wc -l < "$sorted" | tr -d ' ')"
  local idx
  idx=$(( (p * n + 99) / 100 ))
  if [ "$idx" -lt 1 ]; then
    idx=1
  fi
  if [ "$idx" -gt "$n" ]; then
    idx="$n"
  fi
  sed -n "${idx}p" "$sorted"
}

mean_col() {
  local column="$1"
  awk -F '\t' -v c="$column" '{sum += $c; n += 1} END { if (n == 0) { print 0 } else { printf "%.2f", sum/n } }' "$VALUES_FILE"
}

elapsed_p50="$(pctl 2 50)"
elapsed_p95="$(pctl 2 95)"
runner_p50="$(pctl 3 50)"
runner_p95="$(pctl 3 95)"
elapsed_mean="$(mean_col 2)"
runner_mean="$(mean_col 3)"

status_pass="$(awk -F '\t' '$4 == "pass" {c += 1} END {print c+0}' "$VALUES_FILE")"
status_warn="$(awk -F '\t' '$4 == "warn" {c += 1} END {print c+0}' "$VALUES_FILE")"
status_fail="$(awk -F '\t' '$4 == "fail" {c += 1} END {print c+0}' "$VALUES_FILE")"
cache_true="$(awk -F '\t' '$5 == "true" {c += 1} END {print c+0}' "$VALUES_FILE")"
cache_false="$(awk -F '\t' '$5 == "false" {c += 1} END {print c+0}' "$VALUES_FILE")"
cache_unknown="$(awk -F '\t' '$5 != "true" && $5 != "false" {c += 1} END {print c+0}' "$VALUES_FILE")"
retry_true="$(awk -F '\t' '$6 == "true" {c += 1} END {print c+0}' "$VALUES_FILE")"
retry_false="$(awk -F '\t' '$6 == "false" {c += 1} END {print c+0}' "$VALUES_FILE")"
retry_unknown="$(awk -F '\t' '$6 != "true" && $6 != "false" {c += 1} END {print c+0}' "$VALUES_FILE")"
narrow_records="$(awk -F '\t' '$7 ~ /^[0-9]+$/ && $7 <= 3 {c += 1} END {print c+0}' "$VALUES_FILE")"
narrow_elapsed_mean="$(awk -F '\t' '$7 ~ /^[0-9]+$/ && $7 <= 3 {sum += $2; n += 1} END { if (n == 0) { print "0.00" } else { printf "%.2f", sum/n } }' "$VALUES_FILE")"
narrow_runner_mean="$(awk -F '\t' '$7 ~ /^[0-9]+$/ && $7 <= 3 {sum += $3; n += 1} END { if (n == 0) { print "0.00" } else { printf "%.2f", sum/n } }' "$VALUES_FILE")"
narrow_full_scope_count="$(awk -F '\t' '$7 ~ /^[0-9]+$/ && $7 <= 3 && $8 == "full" {c += 1} END {print c+0}' "$VALUES_FILE")"

cat <<REPORT
CI Budget Telemetry Summary
===========================

Records: $count
Lane filter: ${LANE_FILTER:-all}

Runtime (seconds):
- p50: $elapsed_p50
- p95: $elapsed_p95
- mean: $elapsed_mean

Runner minutes:
- p50: $runner_p50
- p95: $runner_p95
- mean: $runner_mean

Narrow-diff slice:
- Narrow-diff records (<=3 changed files): $narrow_records
- Narrow-diff elapsed mean: $narrow_elapsed_mean
- Narrow-diff runner mean: $narrow_runner_mean
- Narrow-diff full-scope count: $narrow_full_scope_count

Status counts:
- pass: $status_pass
- warn: $status_warn
- fail: $status_fail

Cache hit counts:
- true: $cache_true
- false: $cache_false
- unknown/other: $cache_unknown

Retry-used counts:
- true: $retry_true
- false: $retry_false
- unknown/other: $retry_unknown
REPORT
