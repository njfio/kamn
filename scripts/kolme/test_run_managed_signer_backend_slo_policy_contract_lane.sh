#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_managed_signer_backend_slo_policy_contract_lane.sh"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_managed_signer_backend_slo_policy_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/managed_signer_backend_slo_policy_contract_lane.py"
GENERATOR="$ROOT_DIR/scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_managed_signer_backend_slo_policy.py"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
CI_COST_DOC="$ROOT_DIR/docs/ci/ci-cost-and-lane-framework.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected managed-signer backend SLO policy contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -x "$DISPATCHER" ]; then
  echo "expected managed-signer backend SLO policy dispatcher to be executable" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected managed-signer backend SLO policy manifest to exist" >&2
  exit 1
fi

if [ ! -x "$GENERATOR" ]; then
  echo "expected managed-signer backend SLO telemetry generator to be executable" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected managed-signer backend SLO policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected managed-signer backend SLO policy contract lane implementation to exist" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected managed-signer backend SLO policy contract lane runner to be a symlink to shared dispatcher" >&2
  exit 1
fi

if [ "$(readlink "$RUNNER")" != "run_contract_lane_dispatch.sh" ]; then
  echo "expected managed-signer backend SLO policy runner symlink target to be run_contract_lane_dispatch.sh" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected managed-signer backend SLO policy dispatcher to resolve deterministic manifest path" >&2
  exit 1
fi

python3 - "$MANIFEST" <<'PY'
import json
import pathlib
import sys

manifest_path = pathlib.Path(sys.argv[1])
payload = json.loads(manifest_path.read_text(encoding="utf-8"))
if payload.get("schema_version") != "kamn.contract-lane.manifest.v1":
    raise SystemExit("unexpected managed-signer backend SLO policy manifest schema")
if payload.get("lane_id") != "kolme.managed_signer_backend_slo_policy.contract":
    raise SystemExit("unexpected managed-signer backend SLO policy manifest lane_id")
contract_command = payload.get("phases", {}).get("contract")
if contract_command != [
    "python3",
    "scripts/kolme/contracts/managed_signer_backend_slo_policy_contract_lane.py",
]:
    raise SystemExit("unexpected managed-signer backend SLO policy manifest contract command")
PY

required_markers=(
  "check_managed_signer_backend_slo_policy.py"
  "run_managed_signer_backend_slo_policy_contract_lane.sh"
  "kamn.kolme.managed-signer-backend-slo-policy-report.v1"
  "kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1"
  "managed_signer_backend_slo_within_threshold"
  "managed_signer_backend_no_action_required"
  "managed_signer_backend_timeout_rate_threshold_exceeded"
  "managed_signer_backend_unavailable_rate_threshold_exceeded"
  "managed_signer_backend_error_rate_threshold_exceeded"
  "managed_signer_backend_ci_fast_gate_failed"
  "managed_signer_backend_reduce_timeout_burst"
  "managed_signer_backend_failover_endpoint"
  "managed_signer_backend_enable_circuit_breaker"
  "managed_signer_backend_replay_ci_fast_gate"
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
if payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1":
    raise SystemExit("unexpected managed-signer SLO policy contract lane report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected managed-signer SLO policy contract lane final decision GO")
if not isinstance(payload.get("go_policy_report"), str):
    raise SystemExit("expected managed-signer SLO policy GO report path")
if not isinstance(payload.get("no_go_policy_report"), str):
    raise SystemExit("expected managed-signer SLO policy NO-GO report path")
if not isinstance(payload.get("ci_fast_gate_no_go_policy_report"), str):
    raise SystemExit("expected managed-signer SLO policy CI-fast-gate NO-GO report path")
PY

echo "managed-signer backend SLO policy contract lane tests passed."
