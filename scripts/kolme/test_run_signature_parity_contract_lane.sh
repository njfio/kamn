#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_signature_parity_contract_lane.sh"
MATRIX_RUNNER="$ROOT_DIR/scripts/kolme/run_signature_parity_matrix.py"
CHECKER="$ROOT_DIR/scripts/kolme/check_signature_parity_policy.py"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/signature_parity_contract_lane.py"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_signature_parity_contract_lane.json"
FIXTURE="$ROOT_DIR/fixtures/kolme_commit/signature_parity_vectors.json"
ARCH_DOC="$ROOT_DIR/docs/architecture/kolme-live-integration.md"
CI_DOC="$ROOT_DIR/docs/ci/strategy.md"
TMP_REPORT="$(mktemp)"
TMP_POLICY="$(mktemp)"
trap 'rm -f "$TMP_REPORT" "$TMP_POLICY"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected signature parity contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$MATRIX_RUNNER" ]; then
  echo "expected signature parity matrix runner to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected signature parity policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected signature parity contract lane implementation to exist" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected signature parity contract lane manifest to exist" >&2
  exit 1
fi

if [ ! -f "$FIXTURE" ]; then
  echo "expected signature parity vector fixture to exist" >&2
  exit 1
fi

if ! grep -q "run_manifest_lane.sh" "$RUNNER"; then
  echo "expected signature parity contract lane runner to dispatch through run_manifest_lane.sh" >&2
  exit 1
fi

if ! grep -q "kolme_signature_parity_contract_lane.json" "$RUNNER"; then
  echo "expected signature parity contract lane runner to pin deterministic manifest path" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected signature parity manifest schema")
if payload.get("lane_id") != "kolme.signature_parity.contract":
    raise SystemExit("unexpected signature parity manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/signature_parity_contract_lane.py",
]:
    raise SystemExit("unexpected signature parity manifest contract command")
PY

required_impl_markers=(
  "run_signature_parity_matrix.py"
  "check_signature_parity_policy.py"
  "fixtures/kolme_commit/signature_parity_vectors.json"
  "parity_signature_mismatch"
  "Regression: #2299"
)
for marker in "${required_impl_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected signature parity contract implementation marker: $marker" >&2
    exit 1
  fi
done

required_arch_markers=(
  "run_signature_parity_matrix.py"
  "check_signature_parity_policy.py"
  "run_signature_parity_contract_lane.sh"
  "fixtures/kolme_commit/signature_parity_vectors.json"
)
for marker in "${required_arch_markers[@]}"; do
  if ! grep -q "$marker" "$ARCH_DOC"; then
    echo "expected architecture doc signature parity marker: $marker" >&2
    exit 1
  fi
done

required_ci_markers=(
  "test_run_signature_parity_contract_lane.sh"
  "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS=120"
)
for marker in "${required_ci_markers[@]}"; do
  if ! grep -q "$marker" "$CI_DOC"; then
    echo "expected CI strategy signature parity marker: $marker" >&2
    exit 1
  fi
done

bash "$RUNNER" --output-json "$TMP_REPORT" --policy-output-json "$TMP_POLICY" >/dev/null

python3 - "$TMP_REPORT" "$TMP_POLICY" <<'PY'
import json
import pathlib
import sys

report = json.loads(pathlib.Path(sys.argv[1]).read_text(encoding="utf-8"))
policy = json.loads(pathlib.Path(sys.argv[2]).read_text(encoding="utf-8"))
if report.get("schema_version") != "kamn.kolme.signature-parity-matrix-report.v1":
    raise SystemExit("unexpected signature parity matrix report schema")
if report.get("status") != "pass":
    raise SystemExit("expected signature parity matrix status pass")
if report.get("source_contract") != "njfio/kolme_fork-secp256k1-v1":
    raise SystemExit("expected signature parity source contract marker")
cases = report.get("cases", [])
if not isinstance(cases, list) or not cases:
    raise SystemExit("expected signature parity cases in matrix report")
bad_cases = [
    case
    for case in cases
    if isinstance(case, dict)
    and case.get("vector_id") == "kolme_fork_primary_alpha_bad_signature"
]
if len(bad_cases) != 1:
    raise SystemExit("expected one bad signature vector case in matrix report")
bad_case = bad_cases[0]
if bad_case.get("observed_final_decision") != "NO-GO":
    raise SystemExit("expected bad signature vector observed_final_decision NO-GO")
if "parity_signature_mismatch" not in bad_case.get("reason_codes", []):
    raise SystemExit("expected bad signature vector reason_codes to include parity_signature_mismatch")
if policy.get("schema_version") != "kamn.kolme.signature-parity-policy-report.v1":
    raise SystemExit("unexpected signature parity policy report schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signature parity policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected signature parity policy reason_codes to be empty")
PY

echo "signature parity contract lane tests passed."
