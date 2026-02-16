#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_local_signal_secret_hygiene_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_local_signal_secret_hygiene_live_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected local signal/secret hygiene validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected local signal/secret hygiene policy checker script to be executable" >&2
  exit 1
fi

bash "$VALIDATION_SCRIPT" --mode dry-run --output-json "$TMP_REPORT" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected local signal/secret hygiene policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected local signal/secret hygiene policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^local_signal_secret_hygiene_policy_status=verified$'; then
  echo "expected local signal/secret hygiene policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.runtime.local-signal-shutdown-reason-taxonomy.v1$'; then
  echo "expected local signal/secret hygiene policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded$'; then
  echo "expected local signal/secret hygiene policy checker reason codes taxonomy marker" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.local-signal-secret-hygiene-live-policy-report.v1":
    raise SystemExit("unexpected local signal/secret hygiene policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected local signal/secret hygiene policy final_decision=GO")
if payload.get("local_signal_secret_hygiene_policy_status") != "verified":
    raise SystemExit("expected local_signal_secret_hygiene_policy_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.runtime.local-signal-shutdown-reason-taxonomy.v1":
    raise SystemExit("expected deterministic signal shutdown reason taxonomy marker")
if payload.get("reason_codes_csv") != "local_signal_shutdown_path_drift_detected,local_graceful_drain_bypass_detected,ci_local_signal_shutdown_budget_boundary_exceeded":
    raise SystemExit("expected deterministic signal shutdown reason codes taxonomy marker")
PY

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["fallback_secret_fail_closed_reason_code"] = "mismatch-marker"
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
  echo "expected tampered local signal/secret hygiene report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'local_signal_secret_hygiene_policy_secret_reason_code_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered signal/secret hygiene report" >&2
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
if "local_signal_secret_hygiene_policy_secret_reason_code_mismatch" not in reason_codes:
    raise SystemExit("expected parser to recover deterministic signal/secret hygiene reason code")
PY

tampered_signal_shutdown_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_signal_shutdown_report"
python3 - "$tampered_signal_shutdown_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["signal_shutdown_status"] = "drifted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_signal_shutdown_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_signal_shutdown_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_signal_shutdown_code=$?
set -e
rm -f "$tampered_signal_shutdown_report"
if [ "$tampered_signal_shutdown_code" -eq 0 ]; then
  echo "expected signal shutdown drift tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_signal_shutdown_output" | grep -q 'local_signal_shutdown_path_drift_detected'; then
  echo "expected deterministic signal shutdown drift reason code" >&2
  exit 1
fi

tampered_graceful_drain_report="$(mktemp)"
cp "$TMP_REPORT" "$tampered_graceful_drain_report"
python3 - "$tampered_graceful_drain_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["signal_graceful_drain_status"] = "bypass-accepted"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_graceful_drain_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_graceful_drain_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_graceful_drain_code=$?
set -e
rm -f "$tampered_graceful_drain_report"
if [ "$tampered_graceful_drain_code" -eq 0 ]; then
  echo "expected graceful-drain bypass tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_graceful_drain_output" | grep -q 'local_graceful_drain_bypass_detected'; then
  echo "expected deterministic graceful-drain bypass reason code" >&2
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
  echo "expected ci-local signal shutdown budget boundary tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_budget_output" | grep -q 'ci_local_signal_shutdown_budget_boundary_exceeded'; then
  echo "expected deterministic ci-local signal shutdown budget boundary reason code" >&2
  exit 1
fi

echo "local signal/secret hygiene live policy tests passed."
