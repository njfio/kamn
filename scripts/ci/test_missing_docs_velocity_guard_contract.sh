#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
THROUGHPUT_SCRIPT="$ROOT_DIR/scripts/ci/missing_docs_throughput_report_contract.py"
VELOCITY_GUARD_SCRIPT="$ROOT_DIR/scripts/ci/missing_docs_velocity_guard.py"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/kamn_core_missing_docs_velocity_baseline.json"
THRESHOLD_FILE="$ROOT_DIR/.ci/kamn-core-missing-docs-velocity-thresholds.json"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

REPORT_PATH="$TMP_DIR/missing-docs-throughput-report.json"
POLICY_PATH="$TMP_DIR/missing-docs-velocity-policy.json"

if [ ! -x "$THROUGHPUT_SCRIPT" ]; then
  echo "expected throughput report script to be executable" >&2
  exit 1
fi

if [ ! -x "$VELOCITY_GUARD_SCRIPT" ]; then
  echo "expected missing docs velocity guard script to be executable" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected missing docs velocity baseline file" >&2
  exit 1
fi

if [ ! -f "$THRESHOLD_FILE" ]; then
  echo "expected missing docs velocity threshold file" >&2
  exit 1
fi

python3 "$THROUGHPUT_SCRIPT" generate \
  --output-json "$REPORT_PATH" >/dev/null

python3 "$VELOCITY_GUARD_SCRIPT" check \
  --report-file "$REPORT_PATH" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$THRESHOLD_FILE" \
  --output-json "$POLICY_PATH" >"$TMP_DIR/pass.out"

grep -q '^status=pass$' "$TMP_DIR/pass.out"
grep -q '^final_decision=GO$' "$TMP_DIR/pass.out"
grep -q '^reason_key=allowlist_fully_graduated$' "$TMP_DIR/pass.out"
grep -Eq '^commit_delta=[0-9]+$' "$TMP_DIR/pass.out"
grep -q '"schema_version": "kamn.ci.kamn-core-missing-docs-velocity-policy.v1"' "$POLICY_PATH"
python3 - "$POLICY_PATH" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("allowlist_exhausted") is not True:
    raise SystemExit("expected allowlist_exhausted=true for fully graduated baseline")
if payload.get("reason_key") != "allowlist_fully_graduated":
    raise SystemExit("expected terminal allowlist reason key")
PY

MUTATED_BASELINE="$TMP_DIR/mutated-baseline.json"
cp "$BASELINE_FILE" "$MUTATED_BASELINE"
MUTATED_REPORT="$TMP_DIR/mutated-report.json"
cp "$REPORT_PATH" "$MUTATED_REPORT"
python3 - "$MUTATED_BASELINE" "$MUTATED_REPORT" <<'PY'
import json
import sys
from pathlib import Path

baseline_path = Path(sys.argv[1])
report_path = Path(sys.argv[2])

baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
baseline["commit_count"] = 1
baseline_path.write_text(
    json.dumps(baseline, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)

report = json.loads(report_path.read_text(encoding="utf-8"))
report["allowlisted_module_count"] = 1
report_path.write_text(
    json.dumps(report, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY

if python3 "$VELOCITY_GUARD_SCRIPT" check \
  --report-file "$MUTATED_REPORT" \
  --baseline-file "$MUTATED_BASELINE" \
  --threshold-file "$THRESHOLD_FILE" \
  --output-json "$TMP_DIR/fail-policy.json" >"$TMP_DIR/fail.out" 2>&1; then
  echo "expected velocity guard to fail for stagnation" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/fail.out"
grep -q 'stagnation window exceeded' "$TMP_DIR/fail.out"

# Regression: #2127
MUTATED_THRESHOLD="$TMP_DIR/mutated-threshold.json"
cp "$THRESHOLD_FILE" "$MUTATED_THRESHOLD"
python3 - "$MUTATED_THRESHOLD" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["velocity_window_commits"] = 0
path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

if python3 "$VELOCITY_GUARD_SCRIPT" check \
  --report-file "$REPORT_PATH" \
  --baseline-file "$BASELINE_FILE" \
  --threshold-file "$MUTATED_THRESHOLD" \
  --output-json "$TMP_DIR/threshold-fail-policy.json" >"$TMP_DIR/threshold-fail.out" 2>&1; then
  echo "expected velocity guard to fail on invalid threshold configuration" >&2
  exit 1
fi

grep -q '^status=fail$' "$TMP_DIR/threshold-fail.out"
grep -q 'velocity_window_commits must be positive' "$TMP_DIR/threshold-fail.out"

echo "missing docs velocity guard contract tests passed."
