#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_daemon_os_signal_live.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected daemon os signal live validation script to be executable" >&2
  exit 1
fi

validation_output="$(bash "$VALIDATION_SCRIPT" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$validation_output" | grep -q '^status=pass$'; then
  echo "expected daemon os signal live validation pass status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^final_decision=GO$'; then
  echo "expected daemon os signal live validation GO decision marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^os_signal_shutdown_status=verified$'; then
  echo "expected daemon os signal live validation shutdown marker" >&2
  exit 1
fi
if ! printf '%s\n' "$validation_output" | grep -q '^failure_case_status=verified$'; then
  echo "expected daemon os signal live validation fail-closed marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.daemon-os-signal-live-validation.v1":
    raise SystemExit("unexpected daemon os signal live validation schema")
if payload.get("status") != "pass":
    raise SystemExit("expected daemon os signal live validation status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected daemon os signal live validation final_decision=GO")
if payload.get("os_signal_shutdown_status") != "verified":
    raise SystemExit("expected os_signal_shutdown_status=verified")
if payload.get("failure_case_status") != "verified":
    raise SystemExit("expected failure_case_status=verified")
PY

echo "daemon os signal live validation tests passed."
