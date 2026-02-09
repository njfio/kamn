#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/deploy/run_deployment_slo_rollback_lane.sh"
CHECKER="$ROOT_DIR/scripts/deploy/check_deployment_slo_rollback_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected deployment slo/rollback lane script to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected deployment slo/rollback policy checker script to be executable" >&2
  exit 1
fi

go_report="$TMP_DIR/deployment-slo-rollback-go.json"
KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS=true \
bash "$LANE_SCRIPT" --output-json "$go_report" >/dev/null

go_checker_output="$(bash "$CHECKER" --report-file "$go_report")"
if ! printf '%s\n' "$go_checker_output" | grep -q '^status=ok$'; then
  echo "expected deployment slo/rollback policy checker status marker for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^final_decision=GO$'; then
  echo "expected deployment slo/rollback policy checker GO decision for GO report" >&2
  exit 1
fi
if ! printf '%s\n' "$go_checker_output" | grep -q '^failed_checks=none$'; then
  echo "expected deployment slo/rollback policy checker no failed checks for GO report" >&2
  exit 1
fi

rollback_no_go_report="$TMP_DIR/deployment-slo-rollback-no-go.json"
set +e
KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS=true \
KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_ROLLBACK_AUTOMATION_MISSING=true \
bash "$LANE_SCRIPT" --output-json "$rollback_no_go_report" >/dev/null 2>&1
rollback_no_go_code=$?
set -e

if [ "$rollback_no_go_code" -eq 0 ]; then
  echo "expected forced rollback automation missing deployment slo/rollback lane run to fail closed" >&2
  exit 1
fi

rollback_no_go_checker_output="$(bash "$CHECKER" --report-file "$rollback_no_go_report")"
if ! printf '%s\n' "$rollback_no_go_checker_output" | grep -q '^final_decision=NO-GO$'; then
  echo "expected deployment slo/rollback policy checker NO-GO decision for rollback-missing report" >&2
  exit 1
fi
if ! printf '%s\n' "$rollback_no_go_checker_output" | grep -q 'rollback_automation_missing'; then
  echo "expected deployment slo/rollback policy checker failed checks to include rollback_automation_missing" >&2
  exit 1
fi

tampered_report="$TMP_DIR/deployment-slo-rollback-tampered.json"
cp "$go_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_codes"] = ["rollback_automation_missing"]
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$CHECKER" --report-file "$tampered_report" 2>&1)"
tampered_code=$?
set -e

if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered deployment slo/rollback report to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q 'reason_codes mismatch'; then
  echo "expected reason_codes mismatch failure for tampered deployment slo/rollback report" >&2
  exit 1
fi

# Regression: #944
if ! printf '%s\n' "$tampered_output" | grep -q 'expected reason_codes'; then
  echo "expected explicit reason-code mismatch output for deployment slo/rollback regression path" >&2
  exit 1
fi

echo "deployment slo/rollback policy checker tests passed."
