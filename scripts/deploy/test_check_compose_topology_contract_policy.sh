#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CONTRACT_LANE="$ROOT_DIR/scripts/deploy/validate_compose_topology_contract_lane.sh"
POLICY_CHECKER="$ROOT_DIR/scripts/deploy/check_compose_topology_contract_policy.sh"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
TMP_TAMPERED="$(mktemp)"
TMP_TAXONOMY_TAMPERED="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY" "$TMP_TAMPERED" "$TMP_TAXONOMY_TAMPERED"' EXIT

test_harness_require_executable "$CONTRACT_LANE" "expected compose topology contract lane script to be executable"
test_harness_require_executable "$POLICY_CHECKER" "expected compose topology policy checker script to be executable"

bash "$CONTRACT_LANE" --output-json "$TMP_REPORT" --ci-fast-gate PASS >/dev/null

policy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_REPORT" \
    --expected-final-decision GO \
    --ci-fast-gate PASS \
    --output-json "$TMP_POLICY"
)"
if ! printf '%s\n' "$policy_output" | grep -q '^status=ok$'; then
  echo "expected compose topology policy checker status=ok marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^final_decision=GO$'; then
  echo "expected compose topology policy checker final_decision=GO marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^compose_topology_policy_status=verified$'; then
  echo "expected compose topology policy checker status marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_taxonomy_version=kamn.deploy.compose-topology-contract-policy-reason-taxonomy.v1$'; then
  echo "expected compose topology policy checker reason taxonomy marker" >&2
  exit 1
fi
if ! printf '%s\n' "$policy_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected compose topology policy checker reason codes csv marker on pass path" >&2
  exit 1
fi

python3 - "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.deploy.compose-topology-contract-policy-report.v1":
    raise SystemExit("unexpected compose topology policy schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected compose topology policy final_decision=GO")
if payload.get("compose_topology_policy_status") != "verified":
    raise SystemExit("expected compose_topology_policy_status=verified")
if payload.get("reason_taxonomy_version") != "kamn.deploy.compose-topology-contract-policy-reason-taxonomy.v1":
    raise SystemExit("expected deterministic policy reason taxonomy marker")
if payload.get("reason_codes_csv") != "none":
    raise SystemExit("expected deterministic policy reason_codes_csv=none on pass path")
PY

cp "$TMP_REPORT" "$TMP_TAXONOMY_TAMPERED"
python3 - "$TMP_TAXONOMY_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["packaging_reason_taxonomy_version"] = "kamn.deploy.compose-packaging-reason-taxonomy.v0"
payload["packaging_reason_codes_csv"] = "tampered_reason"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_taxonomy_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAXONOMY_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_taxonomy_code=$?
set -e
if [ "$tampered_taxonomy_code" -eq 0 ]; then
  echo "expected packaging taxonomy drift report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'compose_topology_policy_packaging_reason_taxonomy_version_mismatch'; then
  echo "expected deterministic packaging taxonomy mismatch reason for tampered compose topology report" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_taxonomy_output" | grep -q 'compose_topology_policy_packaging_reason_codes_csv_mismatch'; then
  echo "expected deterministic packaging reason-csv mismatch reason for tampered compose topology report" >&2
  exit 1
fi

cp "$TMP_REPORT" "$TMP_TAMPERED"
python3 - "$TMP_TAMPERED" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
payload = json.loads(path.read_text(encoding="utf-8"))
payload["compose_docs_parity_status"] = "tampered"
path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")
PY

set +e
tampered_output="$(
  bash "$POLICY_CHECKER" \
    --report-file "$TMP_TAMPERED" \
    --expected-final-decision GO \
    --ci-fast-gate PASS 2>&1
)"
tampered_code=$?
set -e
if [ "$tampered_code" -eq 0 ]; then
  echo "expected tampered compose topology report to fail policy validation" >&2
  exit 1
fi
if ! printf '%s\n' "$tampered_output" | grep -q 'compose_topology_policy_docs_marker_mismatch'; then
  echo "expected deterministic fail-closed reason for tampered compose topology report" >&2
  exit 1
fi

echo "compose topology policy tests passed."
