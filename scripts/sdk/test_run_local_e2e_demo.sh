#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEMO_SCRIPT="$ROOT_DIR/scripts/sdk/run_local_e2e_demo.sh"

if [ ! -x "$DEMO_SCRIPT" ]; then
  echo "expected local e2e demo runner to be executable" >&2
  exit 1
fi

TMP_OUT="$(mktemp)"
trap 'rm -f "$TMP_OUT"' EXIT

bash "$DEMO_SCRIPT" >"$TMP_OUT"

required_markers=(
  "status=ok"
  "message_id="
  "task_id="
  "artifact_id="
  "escrow_id="
  "requester_balance_before="
  "requester_balance_after="
  "worker_balance_before="
  "worker_balance_after="
  "local e2e demo completed."
)

for marker in "${required_markers[@]}"; do
  if ! grep -Fq "$marker" "$TMP_OUT"; then
    echo "expected local e2e demo output marker '$marker'" >&2
    exit 1
  fi
done

echo "local e2e demo script tests passed."
