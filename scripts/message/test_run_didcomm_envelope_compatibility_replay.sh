#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/message/run_didcomm_envelope_compatibility_replay.py"
FIXTURE="$ROOT_DIR/fixtures/didcomm_envelope_compatibility/replay_cases.json"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected DIDComm envelope compatibility replay runner to be executable" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected DIDComm envelope compatibility replay fixture file to exist" >&2
  exit 1
fi

output="$(
  python3 "$RUNNER" \
    --fixture "$FIXTURE" \
    --output-json "$TMP_REPORT"
)"

if ! printf '%s\n' "$output" | grep -q '^status=pass;'; then
  echo "expected DIDComm envelope compatibility replay runner to pass fixture matrix" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

report_path = pathlib.Path(sys.argv[1])
payload = json.loads(report_path.read_text(encoding="utf-8"))

if payload.get("schema_version") != "kamn.didcomm.envelope-compatibility-report.v1":
    raise SystemExit("unexpected DIDComm envelope compatibility report schema version")

if payload.get("final_decision") != "GO":
    raise SystemExit("expected GO final decision for deterministic DIDComm replay matrix")

cases = payload.get("case_results", [])
if not cases:
    raise SystemExit("expected non-empty DIDComm replay case_results")

expected_case_ids = {
    "vector_s1_plaintext_request",
    "vector_s2_signed_response",
    "vector_f1_missing_recipient_key",
    "vector_f2_unsupported_attachment_mapping",
}
observed_ids = {case.get("case_id") for case in cases}
if observed_ids != expected_case_ids:
    raise SystemExit(f"unexpected DIDComm replay case ids: {sorted(observed_ids)}")
PY

echo "DIDComm envelope compatibility replay runner tests passed."
