#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_managed_signer_backend_slo_telemetry_contract_lane.sh"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/managed_signer_backend_slo_telemetry_contract_lane.py"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected managed-signer backend SLO telemetry contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$GENERATOR" ]; then
  echo "expected managed-signer backend SLO telemetry generator to be executable" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected managed-signer backend SLO telemetry contract lane implementation to exist" >&2
  exit 1
fi

required_markers=(
  "generate_managed_signer_backend_slo_telemetry_bundle.sh"
  "run_managed_signer_backend_slo_telemetry_contract_lane.sh"
  "kamn.kolme.managed-signer-backend-slo-telemetry.v1"
  "managed_signer_backend_timeout_rate_threshold_exceeded"
  "managed_signer_backend_unavailable_rate_threshold_exceeded"
  "managed_signer_backend_error_rate_threshold_exceeded"
  "managed_signer_backend_ci_fast_gate_failed"
  "signer_key_source=managed-external"
  "contracts.required_signer_key_source=managed-external"
)

for docs_file in "$DOC_FILE" "$CI_COST_DOC" "$README_FILE"; do
  for marker in "${required_markers[@]}"; do
    if ! grep -q -- "$marker" "$docs_file"; then
      echo "expected docs parity marker '$marker' in $docs_file" >&2
      exit 1
    fi
  done
done

bash "$RUNNER" --output-json "$TMP_REPORT" >/dev/null

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-contract-report.v1":
    raise SystemExit("unexpected managed-signer SLO contract lane report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected managed-signer SLO contract lane final decision GO")
if not isinstance(payload.get("go_fixture_bundle"), str):
    raise SystemExit("expected managed-signer SLO contract lane GO fixture path")
if not isinstance(payload.get("no_go_fixture_bundle"), str):
    raise SystemExit("expected managed-signer SLO contract lane NO-GO fixture path")
PY

echo "managed-signer backend SLO telemetry contract lane tests passed."
