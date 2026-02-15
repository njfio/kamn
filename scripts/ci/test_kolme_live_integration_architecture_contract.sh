#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARCH_DOC="$ROOT_DIR/docs/architecture/kolme-live-integration.md"
STRATEGY_DOC="$ROOT_DIR/docs/ci/strategy.md"
OPS_DOC="$ROOT_DIR/docs/planning/kolme-devnet-ops.md"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

for required_file in "$ARCH_DOC" "$STRATEGY_DOC" "$OPS_DOC"; do
  if [ ! -f "$required_file" ]; then
    echo "expected documentation file to exist: $required_file" >&2
    exit 1
  fi
done

check_architecture_markers() {
  local doc_path="$1"
  local missing=0
  local required_markers=(
    "Composed Full-Stack E2E Boundary (Task #3433)"
    "validate_local_full_stack_integration_live.sh"
    "run_local_kamn_live_runtime_integration_contract_lane.sh"
    "run_go_no_go_gate_lane.sh"
    "transport_convergence_status"
    "signer_provenance_status"
    "runtime_commit_submission_status"
    "runtime_commit_finality_status"
    "runtime_commit_failure_taxonomy"
    "runtime_provider_client_contract=KolmeRuntimeCommitLiveProvider"
    "local_full_stack_integration_policy_runtime_commit_finality_status_mismatch"
    "release_manifest_missing_required_artifact:local_full_stack_integration"
  )

  for marker in "${required_markers[@]}"; do
    if ! grep -Fq "$marker" "$doc_path"; then
      echo "marker_missing:$marker" >&2
      missing=1
    fi
  done

  if [ "$missing" -ne 0 ]; then
    return 1
  fi
}

check_architecture_markers "$ARCH_DOC"

if ! grep -Fq "docs/architecture/kolme-live-integration.md" "$STRATEGY_DOC"; then
  echo "expected CI strategy to reference docs/architecture/kolme-live-integration.md" >&2
  exit 1
fi
if ! grep -Fq "docs/architecture/kolme-live-integration.md" "$OPS_DOC"; then
  echo "expected Kolme devnet ops plan to reference docs/architecture/kolme-live-integration.md" >&2
  exit 1
fi

BROKEN_ARCH_DOC="$TMP_DIR/kolme-live-integration.broken.md"
cp "$ARCH_DOC" "$BROKEN_ARCH_DOC"
python3 - "$BROKEN_ARCH_DOC" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
path.write_text(
    text.replace("runtime_commit_failure_taxonomy", "runtime_commit_failure_classification", 1),
    encoding="utf-8",
)
PY

if check_architecture_markers "$BROKEN_ARCH_DOC" >"$TMP_DIR/broken.out" 2>"$TMP_DIR/broken.err"; then
  echo "expected architecture marker check to fail for tampered document" >&2
  exit 1
fi
if ! grep -Fq "marker_missing:runtime_commit_failure_taxonomy" "$TMP_DIR/broken.err"; then
  echo "expected deterministic marker_missing reason for tampered architecture marker" >&2
  cat "$TMP_DIR/broken.err" >&2 || true
  exit 1
fi

echo "kolme live integration architecture contract tests passed."
