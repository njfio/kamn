#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEEP_LANE="$ROOT_DIR/scripts/runtime/run_live_network_pilot_deep_lane.sh"
SUMMARY_CHECKER="$ROOT_DIR/scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh"
SUMMARY_GENERATOR="$ROOT_DIR/scripts/runtime/generate_live_network_pilot_artifact_summary.sh"
LIVE_NETWORK_DOC="$ROOT_DIR/docs/planning/live-network-wave.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ ! -x "$DEEP_LANE" || ! -x "$SUMMARY_CHECKER" || ! -x "$SUMMARY_GENERATOR" ]]; then
  echo "expected live-network pilot deep lane scripts to be executable" >&2
  exit 1
fi

if [[ ! -f "$LIVE_NETWORK_DOC" ]]; then
  echo "expected live-network wave planning doc to exist" >&2
  exit 1
fi

summary_json="$TMP_DIR/live-network-pilot-summary.json"
lane_output="$(
  KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS=true \
  bash "$DEEP_LANE" \
    --event-name schedule \
    --skip-suite \
    --max-seconds 120 \
    --output-json "$summary_json"
)"

if ! printf '%s\n' "$lane_output" | grep -q "live-network pilot deep lane tests passed."; then
  echo "expected live-network pilot deep lane success marker" >&2
  exit 1
fi

python3 - "$summary_json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.live-network-pilot-artifact-summary.v1":
    raise SystemExit("unexpected live-network pilot summary schema")
if payload.get("event_name") != "schedule":
    raise SystemExit("expected schedule event in deep-lane summary")
if payload.get("cadence") != "scheduled":
    raise SystemExit("expected scheduled cadence in deep-lane summary")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected deep-lane summary final_decision=GO")
PY

set +e
invalid_event_output="$(
  bash "$DEEP_LANE" \
    --event-name pull_request \
    --output-json "$TMP_DIR/invalid-event.json" 2>&1
)"
invalid_event_code=$?
set -e

if [[ "$invalid_event_code" -eq 0 ]]; then
  echo "expected live-network pilot deep lane to reject non-scheduled cadence" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_event_output" | grep -q "scheduled/manual-only cadence policy"; then
  echo "expected cadence rejection marker for non-scheduled deep-lane execution" >&2
  exit 1
fi

tampered_summary="$TMP_DIR/tampered-summary.json"
bash "$SUMMARY_GENERATOR" \
  --output-file "$tampered_summary" \
  --event-name schedule \
  --cadence scheduled \
  --smoke-status pass \
  --smoke-decision GO \
  --smoke-elapsed-seconds 1 \
  --deep-status fail \
  --deep-decision NO-GO \
  --deep-elapsed-seconds 2 \
  --budget-status within \
  --evidence-complete true \
  --ci-fast-gate PASS >/dev/null

python3 - "$tampered_summary" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["final_decision"] = "GO"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(bash "$SUMMARY_CHECKER" --summary-file "$tampered_summary" 2>&1)"
tampered_code=$?
set -e

if [[ "$tampered_code" -eq 0 ]]; then
  echo "expected tampered live-network pilot summary to fail policy validation" >&2
  exit 1
fi

if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final decision mismatch from live-network pilot summary checker" >&2
  exit 1
fi

if ! grep -q "run_live_network_pilot_deep_lane.sh" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network wave planning doc to reference deep-lane command" >&2
  exit 1
fi

if ! grep -q "kamn.runtime.live-network-pilot-artifact-summary.v1" "$LIVE_NETWORK_DOC"; then
  echo "expected live-network wave planning doc to reference pilot artifact summary schema" >&2
  exit 1
fi

echo "live-network pilot deep contract lane tests passed."
