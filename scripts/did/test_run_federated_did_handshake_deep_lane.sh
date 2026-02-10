#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/did/run_federated_did_handshake_deep_lane.sh"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$DEEP_LANE" ]; then
  echo "expected federated DID handshake deep lane script to be executable" >&2
  exit 1
fi

output="$(
  KAMN_FEDERATED_DID_HANDSHAKE_DEEP_CADENCE=scheduled \
  bash "$DEEP_LANE" \
    --event-name schedule \
    --skip-contract-tests \
    --output-json "$TMP_REPORT"
)"
if ! printf '%s\n' "$output" | grep -q "federated DID handshake deep lane tests passed."; then
  echo "expected federated DID handshake deep lane success marker" >&2
  exit 1
fi

if [ ! -s "$TMP_REPORT" ]; then
  echo "expected federated DID handshake deep lane to produce non-empty report" >&2
  exit 1
fi

if ! grep -Fq "check_federated_did_handshake_deep_policy.sh" "$DEEP_LANE"; then
  echo "expected federated DID handshake deep lane to invoke deep policy checker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text())
if report.get("schema_version") != "kamn.did.federated-handshake.deep-summary.v1":
    raise SystemExit("unexpected federated DID handshake deep summary schema version")
if report.get("event_name") != "schedule":
    raise SystemExit("expected schedule event in federated DID handshake deep summary")
if report.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence in federated DID handshake deep summary")
if report.get("policy_status") != "pass":
    raise SystemExit("expected federated DID handshake deep policy status to pass")
if report.get("final_decision") != "GO":
    raise SystemExit("expected federated DID handshake deep summary to conclude GO")
if report.get("budget_status") != "within":
    raise SystemExit("expected federated DID handshake deep summary budget status within")
PY

set +e
invalid_event_output="$(
  bash "$DEEP_LANE" \
    --event-name pull_request \
    --output-json "$TMP_REPORT" 2>&1
)"
invalid_event_code=$?
set -e

if [[ "$invalid_event_code" -eq 0 ]]; then
  echo "expected federated DID handshake deep lane to reject pull_request cadence" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_event_output" | grep -q "scheduled/manual-only cadence policy"; then
  echo "expected cadence policy rejection marker for federated DID handshake deep lane" >&2
  exit 1
fi

echo "federated DID handshake deep lane script tests passed."
