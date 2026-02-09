#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"
max_runtime="${KAMN_POST_CUTOVER_SLO_MAX_SECONDS:-120}"
if [[ ! "$max_runtime" =~ ^[0-9]+$ ]]; then
  echo "KAMN_POST_CUTOVER_SLO_MAX_SECONDS must be an integer >= 0" >&2
  exit 1
fi

bundle_file="$TMP_DIR/post-cutover-slo-contract.json"

generator_output="$(
  bash "$GENERATOR" \
    --output-file "$bundle_file" \
    --window-minutes 15 \
    --p95-latency-ms 140 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 30 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if ! printf '%s\n' "$generator_output" | grep -q "^final_decision=GO$"; then
  echo "expected post-cutover SLO contract lane bundle decision to be GO" >&2
  exit 1
fi
if ! printf '%s\n' "$generator_output" | grep -q "^reason_key=slo_alert_reason_codes:GO:v1$"; then
  echo "expected post-cutover SLO contract lane bundle reason_key to be GO schema marker" >&2
  exit 1
fi

policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$bundle_file")"
if ! printf '%s\n' "$policy_output" | grep -q "^final_decision=GO$"; then
  echo "expected post-cutover SLO contract lane policy decision to be GO" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q "^reason_key=slo_alert_reason_codes:GO:v1$"; then
  echo "expected post-cutover SLO contract lane policy reason_key to be GO schema marker" >&2
  exit 1
fi

tampered_bundle="$TMP_DIR/post-cutover-slo-alert-drift.json"
cp "$bundle_file" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["alerts"]["alert_keys"] = ["slo.synthetic.alert.drifted"]
payload["alerts"]["total_alerts"] = 1
payload["alerts"]["critical_alerts"] = 1
payload["alerts"]["warning_alerts"] = 0
payload["alerts"]["has_alerts"] = True
payload["alerts"]["highest_severity"] = "CRITICAL"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected post-cutover SLO contract lane alert drift tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "alerts.alert_keys mismatch"; then
  echo "expected explicit alert key drift failure from post-cutover SLO policy checker" >&2
  exit 1
fi

runtime_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$runtime_seconds" -gt "$max_runtime" ]; then
  echo "post-cutover SLO contract lane exceeded runtime budget (${runtime_seconds}s > ${max_runtime}s)" >&2
  exit 1
fi

echo "post-cutover SLO contract lane tests passed."
