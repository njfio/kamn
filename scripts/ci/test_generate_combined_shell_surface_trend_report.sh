#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/ci/generate_combined_shell_surface_trend_report.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$GENERATOR" ]]; then
  echo "expected combined shell-surface trend report generator to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/combined-shell-surface-trend-report.json"
output="$(bash "$GENERATOR" --output-json "$report_file")"

if ! printf '%s\n' "$output" | grep -q '^status=generated$'; then
  echo "expected combined shell-surface trend generator status=generated marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^script_count=[0-9]+$'; then
  echo "expected combined shell-surface trend generator script_count marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^shell_line_total=[0-9]+$'; then
  echo "expected combined shell-surface trend generator shell_line_total marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^rust_line_total=[0-9]+$'; then
  echo "expected combined shell-surface trend generator rust_line_total marker" >&2
  exit 1
fi
if ! printf '%s\n' "$output" | grep -Eq '^shell_to_rust_ratio=[0-9]+(\.[0-9]+)?$'; then
  echo "expected combined shell-surface trend generator shell_to_rust_ratio marker" >&2
  exit 1
fi

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.ci.combined-shell-surface-trend-report.v1":
    raise SystemExit("unexpected combined trend report schema")
if payload.get("status") != "generated":
    raise SystemExit("expected status=generated")

current = payload.get("current", {})
baseline = payload.get("baseline", {})
deltas = payload.get("deltas", {})
script_budget = payload.get("script_budget", {})

for key in ("script_count", "shell_line_total", "rust_line_total"):
    if not isinstance(current.get(key), int) or current.get(key) <= 0:
        raise SystemExit(f"expected positive integer current[{key}]")

if not isinstance(current.get("shell_to_rust_ratio"), float):
    raise SystemExit("expected float current[shell_to_rust_ratio]")
if script_budget.get("status") not in {"pass", "fail"}:
    raise SystemExit("expected script budget status to be pass/fail")

expected_delta_script_count = current["script_count"] - int(baseline["script_count"])
expected_delta_shell_line_total = current["shell_line_total"] - int(baseline["shell_line_total"])
if deltas.get("script_count") != expected_delta_script_count:
    raise SystemExit("script_count delta mismatch")
if deltas.get("shell_line_total") != expected_delta_shell_line_total:
    raise SystemExit("shell_line_total delta mismatch")
PY

set +e
missing_baseline_output="$(bash "$GENERATOR" --combined-baseline-file "$TMP_DIR/missing-baseline.json" --output-json "$TMP_DIR/missing.json" 2>&1)"
missing_baseline_code=$?
set -e
if [[ "$missing_baseline_code" -eq 0 ]]; then
  echo "expected generator to fail when combined baseline file is missing" >&2
  exit 1
fi
if ! printf '%s\n' "$missing_baseline_output" | grep -q 'combined baseline file not found'; then
  echo "expected deterministic missing-baseline failure marker" >&2
  exit 1
fi

echo "combined shell-surface trend report generator tests passed."
