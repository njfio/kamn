#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_signature_parity_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
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

if [ ! -x "$DISPATCHER" ]; then
  echo "expected signature parity contract lane dispatcher to be executable" >&2
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

if [ ! -L "$RUNNER" ]; then
  echo "expected signature parity contract lane runner to be a symlink to shared dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_contract_lane_dispatch.sh" ]; then
  echo "expected signature parity contract lane runner symlink target to be run_contract_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected signature parity contract lane dispatcher to resolve deterministic manifest path" >&2
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
  "parity_recovery_id_mismatch"
  "parity_pubkey_mismatch"
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
expected_negative_vectors = {
    "kolme_fork_primary_alpha_bad_signature": "parity_signature_mismatch",
    "kolme_fork_secondary_beta_bad_recovery": "parity_recovery_id_mismatch",
    "kolme_fork_primary_alpha_bad_pubkey": "parity_pubkey_mismatch",
}
for vector_id, required_reason_code in expected_negative_vectors.items():
    bad_cases = [
        case
        for case in cases
        if isinstance(case, dict)
        and case.get("vector_id") == vector_id
    ]
    if len(bad_cases) != 1:
        raise SystemExit(f"expected one bad parity vector case in matrix report: {vector_id}")
    bad_case = bad_cases[0]
    if bad_case.get("observed_final_decision") != "NO-GO":
        raise SystemExit(f"expected bad parity vector observed_final_decision NO-GO: {vector_id}")
    if required_reason_code not in bad_case.get("reason_codes", []):
        raise SystemExit(
            f"expected bad parity vector reason_codes to include {required_reason_code}: {vector_id}"
        )
if policy.get("schema_version") != "kamn.kolme.signature-parity-policy-report.v1":
    raise SystemExit("unexpected signature parity policy report schema")
if policy.get("final_decision") != "GO":
    raise SystemExit("expected signature parity policy final_decision GO")
if policy.get("reason_codes") != []:
    raise SystemExit("expected signature parity policy reason_codes to be empty")
PY

echo "signature parity contract lane tests passed."
