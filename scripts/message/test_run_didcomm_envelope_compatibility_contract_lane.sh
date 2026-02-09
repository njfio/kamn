#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/message/run_didcomm_envelope_compatibility_contract_lane.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected DIDComm envelope compatibility contract lane script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/didcomm-envelope-compatibility-contract-report.json"
output="$(
  bash "$SCRIPT" \
    --output-json "$report_file" \
    --skip-tests
)"

if ! printf '%s\n' "$output" | grep -q "DIDComm envelope compatibility contract lane tests passed."; then
  echo "expected success output from DIDComm envelope compatibility contract lane" >&2
  exit 1
fi

if [ ! -f "$report_file" ]; then
  echo "expected DIDComm envelope compatibility contract lane to emit report" >&2
  exit 1
fi

if ! grep -q '"schema_version": "kamn.didcomm.envelope-compatibility-report.v1"' "$report_file"; then
  echo "expected DIDComm envelope compatibility report schema marker" >&2
  exit 1
fi

if ! grep -q '"reason_key": "didcomm_envelope_compatibility_reason_codes:GO:v1"' "$report_file"; then
  echo "expected DIDComm envelope compatibility reason key marker in report" >&2
  exit 1
fi

echo "DIDComm envelope compatibility contract lane script tests passed."
