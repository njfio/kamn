#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  bash scripts/canary/check_post_cutover_slo_policy.sh \
    --bundle-file <path>
EOF
}

fail() {
  local message="$1"
  printf '%s\n' "$message" >&2
  exit 1
}

bundle_file=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-file)
      bundle_file="${2:-}"
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown argument: $1"
      ;;
  esac
done

if [[ -z "$bundle_file" ]]; then
  usage
  fail "--bundle-file is required"
fi

if [[ ! -f "$bundle_file" ]]; then
  fail "bundle file not found: $bundle_file"
fi

output="$(
  python3 - "$bundle_file" <<'PY'
import json
import pathlib
import sys


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    sys.exit(1)


bundle_path = pathlib.Path(sys.argv[1])
try:
    payload = json.loads(bundle_path.read_text())
except json.JSONDecodeError as exc:
    fail(f"bundle file is not valid JSON: {exc}")

required_fields = (
    "schema_version",
    "reason_key",
    "generated_at",
    "window_minutes",
    "metrics",
    "decision_reasons",
    "alerts",
    "final_decision",
)
for field in required_fields:
    if field not in payload:
        fail(f"missing bundle field: {field}")

if payload["schema_version"] != "kamn.launch-slo.evidence.v1":
    fail("unexpected post-cutover SLO evidence schema_version")

if not isinstance(payload["window_minutes"], int) or payload["window_minutes"] < 1:
    fail("window_minutes must be an integer >= 1")

metrics = payload["metrics"]
if not isinstance(metrics, dict):
    fail("bundle field 'metrics' must be an object")

for field in (
    "p95_latency_ms",
    "max_p95_latency_ms",
    "error_rate_bps",
    "max_error_rate_bps",
    "delivery_success_bps",
    "min_delivery_success_bps",
    "snapshot_age_seconds",
    "max_snapshot_age_seconds",
    "evidence_complete",
    "ci_fast_gate",
):
    if field not in metrics:
        fail(f"missing metrics field: {field}")

for field in (
    "p95_latency_ms",
    "max_p95_latency_ms",
    "error_rate_bps",
    "max_error_rate_bps",
    "delivery_success_bps",
    "min_delivery_success_bps",
    "snapshot_age_seconds",
    "max_snapshot_age_seconds",
):
    if not isinstance(metrics[field], int):
        fail(f"metrics.{field} must be an integer")

if metrics["max_p95_latency_ms"] < 1:
    fail("metrics.max_p95_latency_ms must be >= 1")
if metrics["max_error_rate_bps"] < 1:
    fail("metrics.max_error_rate_bps must be >= 1")
if metrics["min_delivery_success_bps"] < 1:
    fail("metrics.min_delivery_success_bps must be >= 1")
if metrics["max_snapshot_age_seconds"] < 1:
    fail("metrics.max_snapshot_age_seconds must be >= 1")
if not isinstance(metrics["evidence_complete"], bool):
    fail("metrics.evidence_complete must be a boolean")
if metrics["ci_fast_gate"] not in {"PASS", "FAIL"}:
    fail("metrics.ci_fast_gate must be PASS or FAIL")

decision_reasons: list[str] = []
if metrics["p95_latency_ms"] > metrics["max_p95_latency_ms"]:
    decision_reasons.append("p95-latency-threshold-exceeded")
if metrics["error_rate_bps"] > metrics["max_error_rate_bps"]:
    decision_reasons.append("error-rate-threshold-exceeded")
if metrics["delivery_success_bps"] < metrics["min_delivery_success_bps"]:
    decision_reasons.append("delivery-success-threshold-breached")
if metrics["snapshot_age_seconds"] > metrics["max_snapshot_age_seconds"]:
    decision_reasons.append("stale-snapshot-evidence")
if not metrics["evidence_complete"]:
    decision_reasons.append("incomplete-slo-evidence")
if metrics["ci_fast_gate"] != "PASS":
    decision_reasons.append("ci-fast-gate-failed")

actual_reasons = payload["decision_reasons"]
if not isinstance(actual_reasons, list) or any(not isinstance(item, str) for item in actual_reasons):
    fail("decision_reasons must be an array of strings")
if actual_reasons != decision_reasons:
    fail(
        "decision_reasons mismatch: "
        f"expected {decision_reasons}, found {actual_reasons}"
    )

expected_decision = "GO" if not decision_reasons else "NO-GO"
actual_decision = payload["final_decision"]
if actual_decision not in {"GO", "NO-GO"}:
    fail("final_decision must be GO or NO-GO")

if actual_decision != expected_decision:
    reasons = ", ".join(decision_reasons) if decision_reasons else "all SLO gates satisfied"
    fail(
        "policy decision mismatch: "
        f"expected final_decision={expected_decision}, found {actual_decision}; reasons={reasons}"
    )

expected_reason_key = f"slo_alert_reason_codes:{expected_decision}:v1"
actual_reason_key = payload["reason_key"]
if actual_reason_key != expected_reason_key:
    fail(
        "reason_key mismatch: "
        f"expected {expected_reason_key}, found {actual_reason_key}"
    )

alerts = payload["alerts"]
if not isinstance(alerts, dict):
    fail("bundle field 'alerts' must be an object")
for field in (
    "total_alerts",
    "critical_alerts",
    "warning_alerts",
    "has_alerts",
    "highest_severity",
    "alert_keys",
):
    if field not in alerts:
        fail(f"missing alerts field: {field}")

for field in ("total_alerts", "critical_alerts", "warning_alerts"):
    if not isinstance(alerts[field], int):
        fail(f"alerts.{field} must be an integer")
if not isinstance(alerts["has_alerts"], bool):
    fail("alerts.has_alerts must be a boolean")
if alerts["highest_severity"] not in {"NONE", "WARNING", "CRITICAL"}:
    fail("alerts.highest_severity must be NONE, WARNING, or CRITICAL")
if not isinstance(alerts["alert_keys"], list) or any(
    not isinstance(item, str) for item in alerts["alert_keys"]
):
    fail("alerts.alert_keys must be an array of strings")

reason_to_alert_key = {
    "p95-latency-threshold-exceeded": "slo.latency.p95.threshold_exceeded",
    "error-rate-threshold-exceeded": "slo.error_rate.threshold_exceeded",
    "delivery-success-threshold-breached": "slo.delivery_success.threshold_breached",
    "stale-snapshot-evidence": "slo.snapshot_age.stale",
    "incomplete-slo-evidence": "slo.evidence.incomplete",
    "ci-fast-gate-failed": "slo.ci_fast_gate.failed",
}
reason_to_severity = {
    "p95-latency-threshold-exceeded": "CRITICAL",
    "error-rate-threshold-exceeded": "CRITICAL",
    "delivery-success-threshold-breached": "CRITICAL",
    "stale-snapshot-evidence": "CRITICAL",
    "incomplete-slo-evidence": "WARNING",
    "ci-fast-gate-failed": "WARNING",
}
expected_alert_keys = [reason_to_alert_key[reason] for reason in decision_reasons]
if alerts["alert_keys"] != expected_alert_keys:
    fail(
        "alerts.alert_keys mismatch: "
        f"expected {expected_alert_keys}, found {alerts['alert_keys']}"
    )

expected_critical = sum(
    1 for reason in decision_reasons if reason_to_severity[reason] == "CRITICAL"
)
expected_warning = sum(
    1 for reason in decision_reasons if reason_to_severity[reason] == "WARNING"
)
expected_total = len(expected_alert_keys)
expected_has_alerts = expected_total > 0
expected_highest = "NONE"
if expected_critical > 0:
    expected_highest = "CRITICAL"
elif expected_warning > 0:
    expected_highest = "WARNING"

if alerts["critical_alerts"] != expected_critical:
    fail(
        "alerts.critical_alerts mismatch: "
        f"expected {expected_critical}, found {alerts['critical_alerts']}"
    )
if alerts["warning_alerts"] != expected_warning:
    fail(
        "alerts.warning_alerts mismatch: "
        f"expected {expected_warning}, found {alerts['warning_alerts']}"
    )
if alerts["total_alerts"] != expected_total:
    fail(
        "alerts.total_alerts mismatch: "
        f"expected {expected_total}, found {alerts['total_alerts']}"
    )
if alerts["has_alerts"] != expected_has_alerts:
    fail(
        "alerts.has_alerts mismatch: "
        f"expected {expected_has_alerts}, found {alerts['has_alerts']}"
    )
if alerts["highest_severity"] != expected_highest:
    fail(
        "alerts.highest_severity mismatch: "
        f"expected {expected_highest}, found {alerts['highest_severity']}"
    )

print("status=ok")
print(f"bundle_file={bundle_path}")
print(f"final_decision={actual_decision}")
print(f"reason_key={actual_reason_key}")
print(f"snapshot_age_seconds={metrics['snapshot_age_seconds']}")
print(f"max_snapshot_age_seconds={metrics['max_snapshot_age_seconds']}")
PY
)"

printf '%s\n' "$output"
