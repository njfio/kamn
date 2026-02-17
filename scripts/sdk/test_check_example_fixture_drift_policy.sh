#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT_DIR/scripts/sdk/check_example_fixture_drift_policy.sh"
SHARED_SCRIPT="$ROOT_DIR/scripts/sdk/example_fixture_drift_policy_contract.py"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$SCRIPT" ]; then
  echo "expected sdk example fixture drift policy checker script to be executable" >&2
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

if [ ! -L "$SCRIPT" ]; then
  echo "expected sdk example fixture drift policy checker wrapper to be a symlink" >&2
  exit 1
fi

if [ "$(readlink -f "$SCRIPT")" != "$(readlink -f "$EXEC_DISPATCHER")" ]; then
  echo "expected sdk example fixture drift policy checker wrapper to resolve to shared dispatcher" >&2
  exit 1
fi

if [ ! -x "$SHARED_SCRIPT" ]; then
  echo "expected shared sdk example fixture drift policy checker implementation to be executable" >&2
  exit 1
fi

python3 - "$EXEC_REGISTRY" <<'PY'
import json
import sys
from pathlib import Path

registry = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
entry = registry.get("entries", {}).get("scripts/sdk/check_example_fixture_drift_policy.sh")
if not isinstance(entry, dict):
    raise SystemExit("expected registry entry for sdk example fixture drift policy checker wrapper")
if entry.get("interpreter") != "python3":
    raise SystemExit("expected python3 interpreter for sdk example fixture drift policy checker wrapper")
if entry.get("target") != "scripts/sdk/example_fixture_drift_policy_contract.py":
    raise SystemExit("expected sdk example fixture drift policy checker target in exec registry")
if entry.get("args_prefix") != []:
    raise SystemExit("expected empty args_prefix for sdk example fixture drift policy checker wrapper")
if entry.get("passthrough") is not True:
    raise SystemExit("expected passthrough=true for sdk example fixture drift policy checker wrapper")
PY

pass_report="$TMP_DIR/sdk-example-fixture-drift-report.pass.json"
cat <<'JSON' > "$pass_report"
{
  "schema_version": "kamn.sdk.example-fixture-drift-report.v1",
  "status": "pass",
  "reason_codes": ["none"],
  "fixture": "fixtures/sdk_parity/register_validation_cases.json",
  "snapshot": "fixtures/sdk_parity/register_validation_snapshot.json",
  "case_count": 4,
  "drift_case_ids": []
}
JSON

pass_output="$(bash "$SCRIPT" --report-file "$pass_report")"
if ! printf '%s\n' "$pass_output" | grep -q "^status=ok$"; then
  echo "expected sdk example fixture drift policy checker to accept pass report" >&2
  exit 1
fi

if ! printf '%s\n' "$pass_output" | grep -q "^reason_codes=none$"; then
  echo "expected sdk example fixture drift policy checker to preserve reason code marker" >&2
  exit 1
fi

invalid_schema_report="$TMP_DIR/sdk-example-fixture-drift-report.invalid-schema.json"
cat <<'JSON' > "$invalid_schema_report"
{
  "schema_version": "kamn.sdk.example-fixture-drift-report.v0",
  "status": "pass",
  "reason_codes": ["none"]
}
JSON

set +e
invalid_output="$(bash "$SCRIPT" --report-file "$invalid_schema_report" 2>&1)"
invalid_status=$?
set -e

if [ "$invalid_status" -eq 0 ]; then
  echo "expected sdk example fixture drift policy checker to reject invalid schema report" >&2
  exit 1
fi

if ! printf '%s\n' "$invalid_output" | grep -q "reason=invalid-schema-version"; then
  echo "expected explicit invalid schema reason from sdk example fixture drift policy checker" >&2
  exit 1
fi

echo "sdk example fixture drift policy checker tests passed."
