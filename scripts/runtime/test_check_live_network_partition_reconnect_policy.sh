#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SMOKE_LANE="$ROOT_DIR/scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_network_partition_reconnect_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SMOKE_LANE" ]; then
  echo "expected partition/reconnect smoke lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected partition/reconnect policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/partition-reconnect-report.json"
bash "$SMOKE_LANE" --event-name pull_request --output-json "$report_file" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --max-artifact-age-seconds 900 2>&1
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected partition/reconnect policy checker success status" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected partition/reconnect policy checker GO decision for baseline report" >&2
  exit 1
fi

tampered_report="$TMP_DIR/partition-reconnect-report.tampered.json"
cp "$report_file" "$tampered_report"
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
    --max-artifact-age-seconds 900 2>&1
)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered partition/reconnect report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=GO"; then
  echo "expected partition/reconnect tampered decision mismatch marker" >&2
  exit 1
fi

stale_report="$TMP_DIR/partition-reconnect-report.stale.json"
cp "$report_file" "$stale_report"
python3 - "$stale_report" <<'PY'
import hashlib
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["generated_at_epoch"] = 0

canonical_payload = dict(payload)
canonical_payload.pop("artifact_signature", None)
canonical = json.dumps(canonical_payload, sort_keys=True, separators=(",", ":"))
payload["artifact_signature"] = hashlib.sha256(canonical.encode("utf-8")).hexdigest()

path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
stale_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$stale_report" \
    --max-artifact-age-seconds 1 \
    --now-epoch 5 2>&1
)"
stale_code=$?
set -e

if [ "$stale_code" -eq 0 ]; then
  echo "expected stale partition/reconnect report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$stale_output" | grep -q "matrix artifact is stale"; then
  echo "expected partition/reconnect stale artifact policy marker" >&2
  exit 1
fi

echo "partition/reconnect policy checker tests passed."
