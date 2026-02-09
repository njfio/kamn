#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/governance/run_quorum_attestation_replay_guard_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/governance/check_quorum_attestation_replay_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected governance quorum attestation lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected governance quorum attestation policy checker script to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/governance-quorum-attestation-go.json"
KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS=true \
  bash "$LANE_SCRIPT" --output-file "$go_report" >/dev/null

go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$go_report")"
if [ "$(extract_value "$go_policy_output" "status")" != "ok" ]; then
  echo "expected governance quorum attestation GO policy check status=ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_policy_output" "final_decision")" != "GO" ]; then
  echo "expected governance quorum attestation GO policy check final_decision=GO" >&2
  exit 1
fi

no_go_report="$TMP_DIR/governance-quorum-attestation-no-go.json"
KAMN_GOVERNANCE_QUORUM_ATTESTATION_SKIP_COMMANDS=true \
KAMN_GOVERNANCE_QUORUM_ATTESTATION_FORCE_REPLAY_DETECTED=true \
  bash "$LANE_SCRIPT" --output-file "$no_go_report" >/dev/null

no_go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$no_go_report")"
if [ "$(extract_value "$no_go_policy_output" "final_decision")" != "NO-GO" ]; then
  echo "expected governance quorum attestation NO-GO policy check final_decision=NO-GO" >&2
  exit 1
fi

tampered_report="$TMP_DIR/governance-quorum-attestation-tampered.json"
cp "$no_go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered governance quorum attestation decision to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final_decision mismatch from governance quorum attestation policy checker" >&2
  exit 1
fi

echo "governance quorum attestation replay policy checker tests passed."
