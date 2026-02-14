#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_continuous_runtime_commit_contract_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected continuous runtime commit contract runner to be executable" >&2
  exit 1
fi

run_output="$(bash "$RUNNER" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$run_output" | grep -q '^status=pass$'; then
  echo "expected continuous runtime contract status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^final_decision=GO$'; then
  echo "expected continuous runtime contract GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^continuous_mode_status=verified$'; then
  echo "expected continuous mode marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^finality_recovery_status=verified$'; then
  echo "expected finality recovery marker" >&2
  exit 1
fi
if ! printf '%s\n' "$run_output" | grep -q '^fail_closed_guard_status=verified$'; then
  echo "expected fail-closed guard marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.continuous-runtime-commit.contract.v1":
    raise SystemExit("unexpected continuous runtime contract schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("continuous_mode_status") != "verified":
    raise SystemExit("expected continuous_mode_status=verified")
if payload.get("finality_recovery_status") != "verified":
    raise SystemExit("expected finality_recovery_status=verified")
if payload.get("fail_closed_guard_status") != "verified":
    raise SystemExit("expected fail_closed_guard_status=verified")
PY

set +e
invalid_budget_output="$({ bash "$RUNNER" --max-seconds invalid; } 2>&1)"
invalid_budget_code=$?
set -e
if [ "$invalid_budget_code" -eq 0 ]; then
  echo "expected runner to reject invalid max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$invalid_budget_output" | grep -q 'max-seconds must be an integer'; then
  echo "expected deterministic invalid max-seconds marker" >&2
  exit 1
fi

set +e
zero_budget_output="$({ bash "$RUNNER" --max-seconds 0; } 2>&1)"
zero_budget_code=$?
set -e
if [ "$zero_budget_code" -eq 0 ]; then
  echo "expected runner to reject zero max-seconds" >&2
  exit 1
fi
if ! printf '%s\n' "$zero_budget_output" | grep -q 'max-seconds must be greater than zero'; then
  echo "expected deterministic zero max-seconds marker" >&2
  exit 1
fi

echo "continuous runtime commit contract lane tests passed."
