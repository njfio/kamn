#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNNER="$ROOT_DIR/scripts/kolme/run_local_live_provider_runtime_integration_contract_lane.sh"
CHECKER="$ROOT_DIR/scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/local_live_provider_runtime_integration_contract_lane.py"
DISPATCHER="$ROOT_DIR/scripts/kolme/run_contract_lane_dispatch.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_local_live_provider_runtime_integration_contract_lane.json"
DOC_FILE="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
README_FILE="$ROOT_DIR/README.md"
TMP_REPORT="$(mktemp)"
trap 'rm -f "$TMP_REPORT"' EXIT

if [ ! -x "$RUNNER" ]; then
  echo "expected local live-provider runtime integration contract lane runner to be executable" >&2
  exit 1
fi

if [ ! -L "$RUNNER" ]; then
  echo "expected local live-provider runtime integration contract lane runner to be a symlink dispatcher wrapper" >&2
  exit 1
fi

if [ ! -x "$CHECKER" ]; then
  echo "expected local runtime commit live evidence policy checker to be executable" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected local live-provider runtime integration contract lane implementation to exist" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected local live-provider runtime integration contract lane manifest to exist" >&2
  exit 1
fi

resolved_manifest="$(bash "$DISPATCHER" --lane-wrapper "$(basename "$RUNNER")" --resolve-manifest-path)"
if [ "$resolved_manifest" != "$MANIFEST" ]; then
  echo "expected live-provider runtime integration wrapper to resolve deterministic manifest" >&2
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
    "scripts/kolme/contracts/local_live_provider_runtime_integration_contract_lane.py",
]:
    raise SystemExit("expected local live-provider runtime integration manifest contract command")
PY

required_doc_markers=(
  "run_local_live_provider_runtime_integration_contract_lane.sh"
  "run_local_runtime_commit_live_lane.sh"
  "check_local_runtime_commit_live_evidence_policy.py"
  "provider_client_contract=KolmeRuntimeCommitLiveProvider"
  "provider_client_contract_mismatch"
  "provider_in_memory_reference_detected"
  "live_preflight_failed"
  "live_preflight_timeout"
)

for docs_file in "$DOC_FILE" "$README_FILE"; do
  for marker in "${required_doc_markers[@]}"; do
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
if payload.get("schema_version") != "kamn.kolme.local-live-provider-runtime-integration-contract-report.v1":
    raise SystemExit("unexpected live-provider runtime integration contract report schema")
if payload.get("final_decision") != "GO":
    raise SystemExit("expected live-provider runtime integration contract report final_decision GO")
required_paths = (
    "go_summary_file",
    "go_policy_file",
    "provider_mismatch_summary_file",
    "provider_mismatch_policy_file",
    "unavailable_summary_file",
    "unavailable_policy_file",
)
for key in required_paths:
    if not isinstance(payload.get(key), str):
        raise SystemExit(f"expected string path for {key} in contract report")
PY

echo "local live-provider runtime integration contract lane tests passed."
