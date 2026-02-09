#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/canary/generate_post_cutover_slo_evidence_bundle.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/canary/check_post_cutover_slo_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

assert_eq() {
  local actual="$1"
  local expected="$2"
  local message="$3"
  if [ "$actual" != "$expected" ]; then
    echo "$message: expected '$expected', got '$actual'" >&2
    exit 1
  fi
}

if [ ! -x "$GENERATOR" ]; then
  echo "expected post-cutover SLO evidence bundle generator to be executable" >&2
  exit 1
fi

if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected post-cutover SLO evidence policy checker to be executable" >&2
  exit 1
fi

go_bundle="$TMP_DIR/slo-go.json"
go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$go_bundle" \
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

assert_eq "$(extract_value "$go_generate_output" "status")" "generated" "expected GO SLO bundle generation to succeed"
assert_eq "$(extract_value "$go_generate_output" "final_decision")" "GO" "expected generator to derive GO SLO decision"
assert_eq "$(extract_value "$go_generate_output" "reason_key")" "slo_alert_reason_codes:GO:v1" "expected GO SLO reason-key marker"

go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$go_bundle")"
assert_eq "$(extract_value "$go_policy_output" "status")" "ok" "expected GO SLO bundle policy check to pass"
assert_eq "$(extract_value "$go_policy_output" "final_decision")" "GO" "expected policy check to keep GO SLO decision"
assert_eq "$(extract_value "$go_policy_output" "reason_key")" "slo_alert_reason_codes:GO:v1" "expected GO SLO policy reason-key marker"

python3 - "$go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
alerts = payload.get("alerts")
if not isinstance(alerts, dict):
    raise SystemExit("expected alerts object in GO SLO evidence bundle")
for key in ("total_alerts", "critical_alerts", "warning_alerts", "has_alerts", "alert_keys"):
    if key not in alerts:
        raise SystemExit(f"missing alerts.{key} in GO SLO evidence bundle")
if alerts["total_alerts"] != 0:
    raise SystemExit("expected GO SLO bundle alerts.total_alerts to be 0")
if alerts["has_alerts"] is not False:
    raise SystemExit("expected GO SLO bundle alerts.has_alerts to be false")
if alerts["alert_keys"] != []:
    raise SystemExit("expected GO SLO bundle alerts.alert_keys to be empty")
PY

no_go_bundle="$TMP_DIR/slo-no-go.json"
no_go_generate_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_bundle" \
    --window-minutes 15 \
    --p95-latency-ms 245 \
    --max-p95-latency-ms 200 \
    --error-rate-bps 18 \
    --max-error-rate-bps 25 \
    --delivery-success-bps 9992 \
    --min-delivery-success-bps 9950 \
    --snapshot-age-seconds 360 \
    --max-snapshot-age-seconds 120 \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

assert_eq "$(extract_value "$no_go_generate_output" "final_decision")" "NO-GO" "expected stale/threshold-breached SLO bundle to force NO-GO"
assert_eq "$(extract_value "$no_go_generate_output" "reason_key")" "slo_alert_reason_codes:NO-GO:v1" "expected NO-GO SLO reason-key marker"

no_go_policy_output="$(bash "$POLICY_CHECKER" --bundle-file "$no_go_bundle")"
assert_eq "$(extract_value "$no_go_policy_output" "status")" "ok" "expected NO-GO SLO policy check to pass"
assert_eq "$(extract_value "$no_go_policy_output" "final_decision")" "NO-GO" "expected policy check to keep NO-GO SLO decision"
assert_eq "$(extract_value "$no_go_policy_output" "reason_key")" "slo_alert_reason_codes:NO-GO:v1" "expected NO-GO SLO policy reason-key marker"

python3 - "$no_go_bundle" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
alerts = payload.get("alerts")
if not isinstance(alerts, dict):
    raise SystemExit("expected alerts object in NO-GO SLO evidence bundle")
alert_keys = alerts.get("alert_keys")
if not isinstance(alert_keys, list):
    raise SystemExit("expected alerts.alert_keys to be a list")
required = {
    "slo.latency.p95.threshold_exceeded",
    "slo.snapshot_age.stale",
}
missing = sorted(required.difference(alert_keys))
if missing:
    raise SystemExit(f"missing required NO-GO alert keys: {', '.join(missing)}")
PY

tampered_bundle="$TMP_DIR/slo-tampered.json"
cp "$no_go_bundle" "$tampered_bundle"
python3 - "$tampered_bundle" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text())
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")
PY

set +e
tampered_output="$(bash "$POLICY_CHECKER" --bundle-file "$tampered_bundle" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered SLO decision bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final-decision mismatch error from SLO policy checker" >&2
  exit 1
fi

# Regression: #711
if ! printf '%s\n' "$tampered_output" | grep -q "stale-snapshot-evidence"; then
  echo "expected stale snapshot regression guard to be enforced" >&2
  exit 1
fi

alert_drift_bundle="$TMP_DIR/slo-alert-drift.json"
cp "$no_go_bundle" "$alert_drift_bundle"
python3 - "$alert_drift_bundle" <<'PY'
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
alert_drift_output="$(bash "$POLICY_CHECKER" --bundle-file "$alert_drift_bundle" 2>&1)"
alert_drift_code=$?
set -e

if [ "$alert_drift_code" -eq 0 ]; then
  echo "expected tampered SLO alert-key bundle to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$alert_drift_output" | grep -q "alerts.alert_keys mismatch"; then
  echo "expected explicit alert-key mismatch error from SLO policy checker" >&2
  exit 1
fi

# Regression: #913
if ! printf '%s\n' "$alert_drift_output" | grep -q "expected"; then
  echo "expected fail-closed schema mismatch details for SLO alert-key drift" >&2
  exit 1
fi

echo "post-cutover SLO evidence bundle tests passed."
