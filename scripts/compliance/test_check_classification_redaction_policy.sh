#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LANE_SCRIPT="$ROOT_DIR/scripts/compliance/run_classification_redaction_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/compliance/check_classification_redaction_policy.sh"
SHARED_POLICY="$ROOT_DIR/scripts/compliance/classification_redaction_policy_contract.py"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

extract_value() {
  local output="$1"
  local key="$2"
  printf '%s\n' "$output" | awk -F= -v key="$key" '$1 == key { print $2; exit }'
}

if [ ! -x "$LANE_SCRIPT" ]; then
  echo "expected classification/redaction lane script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected classification/redaction policy checker script to be executable" >&2
  exit 1
fi

if [ ! -x "$EXEC_DISPATCHER" ]; then
  echo "expected shared exec dispatcher script to be executable" >&2
  exit 1
fi

if [ ! -f "$EXEC_REGISTRY" ]; then
  echo "expected exec wrapper registry to exist" >&2
  exit 1
fi

if [ ! -L "$POLICY_CHECKER" ]; then
  echo "expected classification/redaction policy checker wrapper to be a symlink" >&2
  exit 1
fi

if [ "$(readlink -f "$POLICY_CHECKER")" != "$(readlink -f "$EXEC_DISPATCHER")" ]; then
  echo "expected classification/redaction policy checker wrapper to resolve to shared dispatcher" >&2
  exit 1
fi

if [ ! -x "$SHARED_POLICY" ]; then
  echo "expected shared classification/redaction policy checker implementation to be executable" >&2
  exit 1
fi

python3 - "$EXEC_REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
entry = registry.get("entries", {}).get("scripts/compliance/check_classification_redaction_policy.sh")
if not isinstance(entry, dict):
    raise SystemExit("expected registry entry for classification/redaction policy checker wrapper")
if entry.get("interpreter") != "python3":
    raise SystemExit("expected python3 interpreter for classification/redaction policy checker wrapper")
if entry.get("target") != "scripts/compliance/classification_redaction_policy_contract.py":
    raise SystemExit("expected classification/redaction policy checker target in exec registry")
if entry.get("args_prefix") != []:
    raise SystemExit("expected empty args_prefix for classification/redaction policy checker wrapper")
if entry.get("passthrough") is not True:
    raise SystemExit("expected passthrough=true for classification/redaction policy checker wrapper")
PY

go_report="$TMP_DIR/classification-redaction-go.json"
KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
  bash "$LANE_SCRIPT" --output-file "$go_report" >/dev/null

go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$go_report")"
if [ "$(extract_value "$go_policy_output" "status")" != "ok" ]; then
  echo "expected classification/redaction GO policy check status=ok" >&2
  exit 1
fi
if [ "$(extract_value "$go_policy_output" "final_decision")" != "GO" ]; then
  echo "expected classification/redaction GO policy check final_decision=GO" >&2
  exit 1
fi

no_go_report="$TMP_DIR/classification-redaction-no-go.json"
KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS=true \
KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING=true \
  bash "$LANE_SCRIPT" --output-file "$no_go_report" >/dev/null

no_go_policy_output="$(bash "$POLICY_CHECKER" --report-file "$no_go_report")"
if [ "$(extract_value "$no_go_policy_output" "final_decision")" != "NO-GO" ]; then
  echo "expected classification/redaction NO-GO policy check final_decision=NO-GO" >&2
  exit 1
fi

tampered_report="$TMP_DIR/classification-redaction-tampered.json"
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
  echo "expected tampered classification/redaction decision to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q "expected final_decision=NO-GO"; then
  echo "expected explicit final_decision mismatch from classification/redaction policy checker" >&2
  exit 1
fi

echo "classification/redaction policy checker tests passed."
