#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MATRIX_SCRIPT="$ROOT_DIR/scripts/canary/run_launch_canary_matrix.py"
FIXTURE_FILE="$ROOT_DIR/fixtures/launch_canary/critical_path_probe_cases.json"
CONTRACT_LANE="$ROOT_DIR/scripts/canary/run_launch_canary_contract_lane.sh"

report_file="$ROOT_DIR/launch-canary-report.json"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      report_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      cat <<'EOF'
Usage:
  bash scripts/canary/run_launch_canary_deep_lane.sh \
    [--output-json <path>]
EOF
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

mkdir -p "$(dirname "$report_file")"

bash "$CONTRACT_LANE"
python3 "$MATRIX_SCRIPT" --fixture "$FIXTURE_FILE" --output-json "$report_file" >/dev/null

python3 - "$report_file" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text())
cases = payload.get("cases", [])
if not cases:
    raise SystemExit("expected launch canary deep lane report to contain cases")
if payload.get("failed_count") != 0:
    raise SystemExit("expected launch canary deep lane report to have failed_count=0")
has_go = any(case.get("derived_decision") == "GO" for case in cases)
has_no_go = any(case.get("derived_decision") == "NO-GO" for case in cases)
if not has_go or not has_no_go:
    raise SystemExit("expected launch canary deep lane report to contain GO and NO-GO cases")
PY

echo "launch canary deep lane tests passed."
