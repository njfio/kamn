#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_runtime_observability_endpoint_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_runtime_observability_endpoint_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected runtime observability endpoint live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected runtime observability endpoint live policy checker script to be executable" >&2
  exit 1
fi

summary_report="$TMP_DIR/runtime-observability-endpoint-live-summary.json"
policy_report="$TMP_DIR/runtime-observability-endpoint-live-policy.json"
tampered_report="$TMP_DIR/runtime-observability-endpoint-live-summary.tampered.json"

bash "$VALIDATION_SCRIPT" --output-json "$summary_report" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected runtime observability endpoint live policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected runtime observability endpoint live policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^runtime_observability_policy_status=verified$'; then
  echo "expected runtime observability endpoint live policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.observability-endpoint-live-policy-report.v1":
    raise SystemExit("unexpected runtime observability endpoint policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("runtime_observability_policy_status") != "verified":
    raise SystemExit("expected runtime_observability_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
PY

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "NO-GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-observability-endpoint-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered runtime observability endpoint report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'runtime_observability_policy_final_decision_mismatch'; then
  echo "expected deterministic mismatch reason code for tampered policy validation" >&2
  exit 1
fi

echo "runtime observability endpoint live policy checker tests passed."
