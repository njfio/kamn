#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_retry_diagnostics_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local retry/diagnostics live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local retry/diagnostics live policy checker script to be executable" >&2
  exit 1
fi

summary_report="$TMP_DIR/runtime-local-retry-diagnostics-summary.json"
policy_report="$TMP_DIR/runtime-local-retry-diagnostics-policy.json"
tampered_report="$TMP_DIR/runtime-local-retry-diagnostics-summary.tampered.json"

bash "$VALIDATION_SCRIPT" --mode dry-run --max-seconds 60 --output-json "$summary_report" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local retry/diagnostics policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local retry/diagnostics policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_retry_diagnostics_policy_status=verified$'; then
  echo "expected local retry/diagnostics policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-retry-diagnostics-live-policy-report.v1":
    raise SystemExit("unexpected local retry/diagnostics policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected local retry/diagnostics policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local retry/diagnostics policy final_decision=GO")
if payload.get("local_retry_diagnostics_policy_status") != "verified":
    raise SystemExit("expected local retry/diagnostics policy marker")
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
payload.pop("correlation_diagnostics_status", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/runtime-local-retry-diagnostics-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered local retry/diagnostics report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'missing required report fields: correlation_diagnostics_status'; then
  echo "expected deterministic missing field reason marker for tampered local retry/diagnostics report" >&2
  exit 1
fi

set +e
ci_failed_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL \
    --output-json "$TMP_DIR/runtime-local-retry-diagnostics-policy.ci-failed.json" 2>&1
)"
ci_failed_code=$?
set -e
if [ "$ci_failed_code" -eq 0 ]; then
  echo "expected local retry/diagnostics policy checker to fail when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$ci_failed_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed reason marker" >&2
  exit 1
fi

echo "local retry/diagnostics live policy checker tests passed."
