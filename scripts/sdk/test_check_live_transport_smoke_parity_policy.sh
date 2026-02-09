#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_SCRIPT="$ROOT_DIR/scripts/sdk/run_live_transport_smoke_parity_lane.sh"
CHECKER="$ROOT_DIR/scripts/sdk/check_live_transport_smoke_parity_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SMOKE_SCRIPT" ]; then
  echo "expected sdk smoke parity lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected sdk smoke parity policy checker to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/sdk-smoke-parity-go.json"
KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true bash "$SMOKE_SCRIPT" --output-json "$go_report" >/dev/null

go_checker_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_checker_output" | grep -q '^final_decision=GO$'; then
  echo "expected sdk smoke parity checker GO decision for go report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^failed_checks=none$'; then
  echo "expected sdk smoke parity checker to report no failed checks on go report" >&2
  exit 1
fi

runtime_report="$TMP_DIR/sdk-smoke-parity-runtime-no-go.json"
set +e
KAMN_SDK_SMOKE_PARITY_SKIP_COMMANDS=true \
KAMN_SDK_SMOKE_PARITY_FAKE_DELAY_SECONDS=1 \
KAMN_SDK_SMOKE_PARITY_MAX_SECONDS=0 \
bash "$SMOKE_SCRIPT" --output-json "$runtime_report" >/dev/null 2>&1
runtime_code=$?
set -e

if [ "$runtime_code" -eq 0 ]; then
  echo "expected runtime budget sdk smoke parity run to fail closed" >&2
  exit 1
fi

runtime_checker_output="$(bash "$CHECKER" --report-file "$runtime_report")"
if ! printf '%s\n' "$runtime_checker_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected sdk smoke parity checker NO-GO decision for runtime budget failure report" >&2
  exit 1
fi
if ! printf '%s\n' "$runtime_checker_output" | grep -q 'runtime_budget_exceeded'; then
  echo "expected sdk smoke parity checker failed checks to include runtime_budget_exceeded" >&2
  exit 1
fi

tampered_report="$TMP_DIR/sdk-smoke-parity-tampered.json"
cp "$go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["runtime_budget_exceeded"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered sdk smoke parity report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q 'reason_codes mismatch'; then
  echo "expected reason_codes mismatch failure for tampered sdk smoke parity report" >&2
  exit 1
fi

# Regression: #938
if ! printf '%s\n' "$tampered_output" | grep -q 'expected reason_codes'; then
  echo "expected explicit reason code mismatch output for sdk smoke parity regression path" >&2
  exit 1
fi

echo "sdk live transport smoke parity policy checker tests passed."
