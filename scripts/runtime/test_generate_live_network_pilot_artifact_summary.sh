#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
GENERATOR="$ROOT_DIR/scripts/runtime/generate_live_network_pilot_artifact_summary.sh"
CHECKER="$ROOT_DIR/scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [[ ! -x "$GENERATOR" ]]; then
  echo "expected live-network pilot artifact summary generator to be executable" >&2
  exit 1
fi

if [[ ! -x "$CHECKER" ]]; then
  echo "expected live-network pilot artifact summary checker to be executable" >&2
  exit 1
fi

go_summary="$TMP_DIR/live-network-go-summary.json"
go_output="$(
  bash "$GENERATOR" \
    --output-file "$go_summary" \
    --event-name schedule \
    --cadence scheduled \
    --smoke-status pass \
    --smoke-decision GO \
    --smoke-elapsed-seconds 4 \
    --deep-status pass \
    --deep-decision GO \
    --deep-elapsed-seconds 8 \
    --budget-status within \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if [[ "$(extract_value "$go_output" "status")" != "generated" ]]; then
  echo "expected GO summary generation status=generated" >&2
  exit 1
fi
if [[ "$(extract_value "$go_output" "final_decision")" != "GO" ]]; then
  echo "expected GO summary generation final_decision=GO" >&2
  exit 1
fi

go_check_output="$(bash "$CHECKER" --summary-file "$go_summary")"
if [[ "$(extract_value "$go_check_output" "status")" != "ok" ]]; then
  echo "expected GO summary checker status=ok" >&2
  exit 1
fi
if [[ "$(extract_value "$go_check_output" "final_decision")" != "GO" ]]; then
  echo "expected GO summary checker final_decision=GO" >&2
  exit 1
fi

go_summary_copy="$TMP_DIR/live-network-go-summary-copy.json"
bash "$GENERATOR" \
  --output-file "$go_summary_copy" \
  --event-name schedule \
  --cadence scheduled \
  --smoke-status pass \
  --smoke-decision GO \
  --smoke-elapsed-seconds 4 \
  --deep-status pass \
  --deep-decision GO \
  --deep-elapsed-seconds 8 \
  --budget-status within \
  --evidence-complete true \
  --ci-fast-gate PASS >/dev/null

if ! cmp -s "$go_summary" "$go_summary_copy"; then
  echo "expected deterministic summary output for identical live-network pilot inputs" >&2
  exit 1
fi

no_go_summary="$TMP_DIR/live-network-no-go-summary.json"
no_go_output="$(
  bash "$GENERATOR" \
    --output-file "$no_go_summary" \
    --event-name workflow_dispatch \
    --cadence manual \
    --smoke-status pass \
    --smoke-decision GO \
    --smoke-elapsed-seconds 4 \
    --deep-status fail \
    --deep-decision NO-GO \
    --deep-elapsed-seconds 12 \
    --budget-status exceeded \
    --evidence-complete true \
    --ci-fast-gate PASS
)"

if [[ "$(extract_value "$no_go_output" "final_decision")" != "NO-GO" ]]; then
  echo "expected NO-GO summary generation for failed deep lane input" >&2
  exit 1
fi

no_go_check_output="$(bash "$CHECKER" --summary-file "$no_go_summary")"
if [[ "$(extract_value "$no_go_check_output" "final_decision")" != "NO-GO" ]]; then
  echo "expected NO-GO summary checker decision for failed deep lane input" >&2
  exit 1
fi

tampered_summary="$TMP_DIR/tampered-live-network-summary.json"
cp "$no_go_summary" "$tampered_summary"
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
tampered_output="$(bash "$CHECKER" --summary-file "$tampered_summary" 2>&1)"
tampered_code=$?
set -e

if [[ "$tampered_code" -eq 0 ]]; then
  echo "expected tampered live-network pilot summary to fail policy validation" >&2
  exit 1
fi

# Regression: #829
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected final decision mismatch guard for tampered live-network summary" >&2
  exit 1
fi

echo "live-network pilot artifact summary generator tests passed."
