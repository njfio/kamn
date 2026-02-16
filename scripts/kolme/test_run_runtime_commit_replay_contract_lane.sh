#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CONTRACT_LANE="$ROOT_DIR/scripts/kolme/run_runtime_commit_replay_contract_lane.sh"
MANIFEST="$ROOT_DIR/scripts/framework/manifests/kolme_runtime_commit_replay_contract_lane.json"
CONTRACT_IMPL="$ROOT_DIR/scripts/kolme/contracts/runtime_commit_replay_contract_lane.py"
ROADMAP_DOC="$ROOT_DIR/docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC="$ROOT_DIR/docs/foundation/release-gonogo-checklist.md"

if [ ! -x "$CONTRACT_LANE" ]; then
  echo "expected runtime commit replay contract lane script to be executable" >&2
  exit 1
fi

if ! grep -q "scripts/framework/run_manifest_lane.sh" "$CONTRACT_LANE"; then
  echo "expected runtime commit replay contract lane to dispatch through manifest wrapper" >&2
  exit 1
fi

if [ ! -f "$MANIFEST" ]; then
  echo "expected runtime commit replay contract lane manifest to exist" >&2
  exit 1
fi

if [ ! -f "$CONTRACT_IMPL" ]; then
  echo "expected runtime commit replay contract lane implementation to exist" >&2
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
    "scripts/kolme/contracts/runtime_commit_replay_contract_lane.py",
]:
    raise SystemExit("expected runtime commit replay manifest contract command")
PY

if ! grep -q "run_runtime_commit_replay_contract_lane.sh" "$ROADMAP_DOC"; then
  echo "expected Kolme integration roadmap to reference runtime commit replay lane command" >&2
  exit 1
fi

required_markers=(
  "recovery_reason_taxonomy_version"
  "recovery_reason_codes_csv"
  "recovery_reason_codes_value"
  "retransmission_evidence_contract_version"
  "nonce_idempotency_contract_version"
  "recovery_nonce_not_monotonic"
  "recovery_payload_hash_mismatch"
  "recovery_receipt_not_final"
  "recovery_replay_detected"
)
for marker in "${required_markers[@]}"; do
  if ! grep -q "$marker" "$CONTRACT_IMPL"; then
    echo "expected runtime commit replay contract lane marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$ROADMAP_DOC"; then
    echo "expected Kolme roadmap marker: $marker" >&2
    exit 1
  fi
  if ! grep -q "$marker" "$GONOGO_DOC"; then
    echo "expected release go/no-go marker: $marker" >&2
    exit 1
  fi
done

lane_output="$(bash "$CONTRACT_LANE")"
if ! printf '%s\n' "$lane_output" | grep -q "Kolme runtime commit replay contract lane tests passed."; then
  echo "expected runtime commit replay contract lane success marker" >&2
  exit 1
fi

echo "runtime commit replay contract lane script tests passed."
