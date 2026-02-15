#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VALIDATION_SCRIPT="$ROOT_DIR/scripts/runtime/validate_unified_api_observability_local_heavy_live.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/runtime/check_unified_api_observability_local_heavy_live_policy.sh"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$VALIDATION_SCRIPT" ]; then
  echo "expected unified API-observability local-heavy validation script to be executable" >&2
  exit 1
fi
if [ ! -x "$POLICY_CHECKER" ]; then
  echo "expected unified API-observability local-heavy policy checker script to be executable" >&2
  exit 1
fi

report_file="$TMP_DIR/unified-api-observability-local-heavy-summary.json"
bash "$VALIDATION_SCRIPT" \
  --mode dry-run \
  --ci-fast-gate PASS \
  --output-json "$report_file" >/dev/null

policy_report="$TMP_DIR/unified-api-observability-local-heavy-policy.json"
policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$policy_report"
)"
for marker in \
  '^status=ok$' \
  '^final_decision=GO$' \
  '^unified_api_observability_local_heavy_policy_status=verified$' \
  '^reason_codes=none$'; do
  if ! printf '%s\n' "$policy_output" | grep -q "$marker"; then
    echo "expected unified API-observability local-heavy policy marker: $marker" >&2
    exit 1
  fi
done

python3 - "$policy_report" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.runtime.unified-api-observability-local-heavy-live-policy-report.v1":
    raise SystemExit("unexpected unified API-observability local-heavy policy schema")
if payload.get("status") != "pass":
    raise SystemExit("expected policy status=pass")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected policy final_decision=GO")
if payload.get("unified_api_observability_local_heavy_policy_status") != "verified":
    raise SystemExit("expected unified_api_observability_local_heavy_policy_status=verified")
PY

tampered_report="$TMP_DIR/unified-api-observability-local-heavy-summary.tampered.json"
cp "$report_file" "$tampered_report"
python3 - "$tampered_report" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["compatibility_matrix_status"] = "missing"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$tampered_report" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_DIR/unified-api-observability-local-heavy-policy.tampered.json" 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered unified API-observability local-heavy report to fail policy checker" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch'; then
  echo "expected deterministic compatibility matrix drift reason code for unified API-observability local-heavy policy checker" >&2
  exit 1
fi

set +e
fast_gate_fail_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$report_file" \
    --expected-final-decision GO \
    --ci-fast-gate FAIL 2>&1
)"
fast_gate_fail_code=$?
set -e
if [ "$fast_gate_fail_code" -eq 0 ]; then
  echo "expected unified API-observability local-heavy policy checker to fail closed when ci-fast-gate=FAIL" >&2
  exit 1
fi
if ! printf '%s\n' "$fast_gate_fail_output" | grep -q 'ci_fast_gate_failed'; then
  echo "expected deterministic ci_fast_gate_failed marker for unified API-observability local-heavy policy checker" >&2
  exit 1
fi

echo "unified API-observability local-heavy policy checker tests passed."
