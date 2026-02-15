#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_structured_logging_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_structured_logging_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected structured logging live validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected structured logging live policy checker script to be executable" >&2
  exit 1
fi

summary_report="$TMP_DIR/structured-logging-live-summary.json"
policy_report="$TMP_DIR/structured-logging-live-policy.json"
tampered_report="$TMP_DIR/structured-logging-live-summary.tampered.json"

bash "$VALIDATION_SCRIPT" --output-json "$summary_report" >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$summary_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected structured logging live policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected structured logging live policy checker GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^structured_logging_policy_status=verified$'; then
  echo "expected structured logging live policy checker status marker" >&2
  exit 1
fi

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.structured-logging-live-policy-report.v1":
    raise SystemExit("unexpected structured logging live policy report schema")
if payload.get("status") != "pass":
    raise SystemExit("expected status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected final_decision=GO")
if payload.get("structured_logging_policy_status") != "verified":
    raise SystemExit("expected structured_logging_policy_status=verified")
if payload.get("reason_codes") != ["none"]:
    raise SystemExit("expected policy checker success reason code ['none']")
if payload.get("reason_taxonomy_version") != "kamn.runtime.structured-logging-live-fail-closed-reason-taxonomy.v1":
    raise SystemExit("expected deterministic reason_taxonomy_version marker")
PY

cp "$summary_report" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload.pop("structured_logging_contract_status", None)
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/structured-logging-live-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered structured logging report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'structured_logging_policy_marker_missing:structured_logging_contract_status'; then
  echo "expected deterministic marker-missing reason code" >&2
  exit 1
fi

tampered_taxonomy_report="$TMP_DIR/structured-logging-live-summary.taxonomy.tampered.json"
cp "$summary_report" "$tampered_taxonomy_report"
python3 - "$tampered_taxonomy_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["reason_taxonomy_version"] = "tampered-taxonomy"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_taxonomy_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/structured-logging-live-policy.taxonomy.tampered.json" 2>&1
)"
tampered_taxonomy_code=$?
set -e
if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected reason-taxonomy tamper to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'structured_logging_policy_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic reason-taxonomy mismatch marker" >&2
  exit 1
fi

echo "structured logging live policy checker tests passed."
