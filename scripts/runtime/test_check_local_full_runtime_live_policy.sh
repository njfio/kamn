#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_full_runtime_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_full_runtime_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
TMP_TAMPERED_TRANSPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_TAMPERED_TRANSPORT"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local full-runtime validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local full-runtime policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --ci-fast-gate PASS --output-json "$TMP_REPORT" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local full-runtime policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local full-runtime policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_full_runtime_policy_status=verified$'; then
  echo "expected local full-runtime policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1$'; then
  echo "expected local full-runtime policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded$'; then
  echo "expected local full-runtime policy checker reason codes taxonomy marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-full-runtime-live-policy-report.v1":
    raise SystemExit("unexpected local full-runtime policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local full-runtime policy final_decision=GO")
if payload.get("local_full_runtime_policy_status") != "verified":
    raise SystemExit("expected local_full_runtime_policy_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.local-full-runtime-error-reason-taxonomy.v1":
    raise SystemExit("expected deterministic local full-runtime reason taxonomy marker")
if payload.get("reason_codes_csv") != "runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded":
    raise SystemExit("expected deterministic local full-runtime reason codes taxonomy marker")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fast_gate_exclusion_status"] = "mismatch-marker"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered local full-runtime report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'local_full_runtime_policy_fast_gate_exclusion_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered local full-runtime report" >&2
  exit 1
fi

python3 - "$tampered_output" <<'PY'
import sys

output = sys.argv[1]
failed_checks = ""
for line in output.splitlines():
    if line.startswith("failed_checks="):
        failed_checks = line.split("=", 1)[1]
        break
reason_codes = [entry for entry in failed_checks.split(",") if entry]
if "local_full_runtime_policy_fast_gate_exclusion_mismatch" not in reason_codes:
    raise SystemExit("expected parser to recover deterministic local full-runtime reason code")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED_TRANSPORT"
python3 - "$TMP_TAMPERED_TRANSPORT" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["runtime_transport_mode"] = "in_memory_simulation"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_transport_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED_TRANSPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_transport_code=$?
set -e
if [ "$tampered_transport_code" -eq 0 ]; then
  echo "expected transport-mode tampered local full-runtime report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_transport_output" | grep -q 'runtime_fallback_classification_unstable'; then
  echo "expected deterministic runtime fallback classification reason for local full-runtime report" >&2
  exit 1
fi

tampered_shutdown_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_shutdown_report"
python3 - "$tampered_shutdown_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["full_runtime_shutdown_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_shutdown_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_shutdown_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_shutdown_code=$?
set -e
rm -f "$tampered_shutdown_report"
if [ "$tampered_shutdown_code" -eq 0 ]; then
  echo "expected shutdown-status tampered local full-runtime report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_shutdown_output" | grep -q 'runtime_full_shutdown_gate_drift_detected'; then
  echo "expected deterministic runtime shutdown gate drift reason for local full-runtime report" >&2
  exit 1
fi

tampered_budget_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_budget_report"
python3 - "$tampered_budget_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["max_seconds"] = 241
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_budget_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_budget_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_budget_code=$?
set -e
rm -f "$tampered_budget_report"
if [ "$tampered_budget_code" -eq 0 ]; then
  echo "expected ci-local runtime extraction budget tampered report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_budget_output" | grep -q 'ci_local_runtime_extraction_budget_boundary_exceeded'; then
  echo "expected deterministic ci-local runtime extraction budget reason code" >&2
  exit 1
fi

echo "local full-runtime live policy tests passed."
