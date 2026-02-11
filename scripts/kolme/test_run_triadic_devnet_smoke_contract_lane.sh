#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_triadic_devnet_smoke_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_triadic_devnet_smoke_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/triadic_devnet_smoke_contract_lane.py"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected triadic devnet smoke contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected triadic devnet smoke contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected triadic devnet smoke contract lane manifest to exist" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/triadic_devnet_smoke_contract_lane.py",
]:
    raise SystemExit("expected triadic devnet smoke manifest contract command")
PY

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected triadic devnet smoke contract implementation to exist" >&2
  exit 1
fi

required_coverage_markers=(
  "run_triadic_devnet_smoke.sh"
  "validate_triadic_devnet_smoke.py"
  "fixtures/kolme_compatibility/devnet_smoke_markers.json"
)
for marker in "${required_coverage_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected triadic devnet smoke contract implementation to include coverage marker: $marker" >&2
    exit 1
  fi
done

lane_output="$(bash "$CONTRACT_LANE" --output-json "$TMP_REPORT")"
if ! printf '%s\n' "$lane_output" | grep -q "triadic devnet smoke contract lane tests passed."; then
  echo "expected triadic devnet smoke contract lane success marker" >&2
  exit 1
fi

python3 - "$TMP_REPORT" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
if payload.get("schema_version") != "kamn.kolme.triadic-devnet-smoke-validation-report.v1":
    raise SystemExit("unexpected triadic devnet smoke report schema")
if payload.get("final_decision") != "PASS":
    raise SystemExit("expected triadic devnet smoke report to pass")
PY

echo "triadic devnet smoke contract lane script tests passed."
