#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/signer/run_signer_incident_recovery_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected signer incident recovery lane script to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/signer-incident-recovery-go.json"
go_output="$(
  KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS=true \
    bash "$LANE_SCRIPT" --output-json "$go_report"
)"

if ! printf '%s\n' "$go_output" | grep -q '^status=pass$'; then
  echo "expected signer incident recovery lane to report pass status for deterministic GO path" >&2
  exit 1
fi
if ! printf '%s\n' "$go_output" | grep -q '^final_decision=GO$'; then
  echo "expected signer incident recovery lane to report GO decision for deterministic GO path" >&2
  exit 1
fi

python3 - "$go_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.signer.incident-recovery-report.v1":
    raise SystemExit("unexpected schema_version for signer incident recovery report")
if payload.get("status") != "pass":
    raise SystemExit("expected pass status for signer incident recovery GO path")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for signer incident recovery GO path")
if payload.get("reason_codes") != []:
    raise SystemExit("expected empty reason_codes for signer incident recovery GO path")
PY

no_go_report="$TMP_DIR/signer-incident-recovery-no-go.json"
set +e
no_go_output="$(
  KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS=true \
  KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP=true \
    bash "$LANE_SCRIPT" --output-json "$no_go_report" 2>&1
)"
no_go_code=$?
set -e

if [ "$no_go_code" -eq 0 ]; then
  echo "expected forced runbook-gap signer incident recovery lane run to fail closed" >&2
  exit 1
fi
if ! printf '%s\n' "$no_go_output" | grep -q 'incident_runbook_step_missing'; then
  echo "expected forced runbook-gap signer incident recovery lane reason code marker" >&2
  exit 1
fi

python3 - "$no_go_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("status") != "fail":
    raise SystemExit("expected fail status for signer incident recovery runbook-gap path")
if payload.get("final_decision") != "NO-GO":
    raise SystemExit("expected NO-GO final decision for signer incident recovery runbook-gap path")
if "incident_runbook_step_missing" not in payload.get("reason_codes", []):
    raise SystemExit("expected incident_runbook_step_missing reason code in signer incident recovery runbook-gap path")
PY

echo "signer incident recovery lane script tests passed."
