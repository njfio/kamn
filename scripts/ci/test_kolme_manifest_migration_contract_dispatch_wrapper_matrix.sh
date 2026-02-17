#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh"
EXEC_DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
EXEC_REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected Kolme manifest-migration dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi
if [ ! -x "$EXEC_DISPATCHER" ]; then
  echo "expected exec wrapper dispatcher to be executable: $EXEC_DISPATCHER" >&2
  exit 1
fi
if [ ! -f "$EXEC_REGISTRY" ]; then
  echo "expected exec wrapper registry to exist: $EXEC_REGISTRY" >&2
  exit 1
fi

wrapper_scripts=(
  "test_kolme_tranche1_manifest_migration_contract.sh"
  "test_kolme_runtime_nonce_manifest_migration_contract.sh"
  "test_kolme_version_matrix_manifest_migration_contract.sh"
  "test_kolme_profile_selftest_portability_manifest_migration_contract.sh"
  "test_kolme_runtime_triadic_bootstrap_e2e_manifest_migration_contract.sh"
  "test_kolme_bootstrap_conformance_runtime_process_manifest_migration_contract.sh"
  "test_kolme_parity_demo_real_process_manifest_migration_contract.sh"
)
group_keys=(
  "tranche1"
  "runtime_nonce"
  "version_matrix"
  "profile_selftest_portability"
  "runtime_triadic_bootstrap_e2e"
  "bootstrap_conformance_runtime_process"
  "parity_demo_real_process"
)

for i in "${!wrapper_scripts[@]}"; do
  wrapper_path="$ROOT_DIR/scripts/ci/${wrapper_scripts[$i]}"
  expected_group="${group_keys[$i]}"

  if [ ! -x "$wrapper_path" ]; then
    echo "expected manifest-migration wrapper to be executable: $wrapper_path" >&2
    exit 1
  fi

  if [ ! -L "$wrapper_path" ]; then
    echo "expected wrapper to be symlinked through shared exec dispatcher: ${wrapper_scripts[$i]}" >&2
    exit 1
  fi

  if [ "$(readlink -f "$wrapper_path")" != "$(readlink -f "$EXEC_DISPATCHER")" ]; then
    echo "expected wrapper to resolve to shared exec dispatcher: ${wrapper_scripts[$i]}" >&2
    exit 1
  fi

  python3 - "$EXEC_REGISTRY" "scripts/ci/${wrapper_scripts[$i]}" "$expected_group" <<'PY'
import json
import sys
from pathlib import Path

registry_path = Path(sys.argv[1])
wrapper_rel = sys.argv[2]
expected_group = sys.argv[3]

payload = json.loads(registry_path.read_text(encoding="utf-8"))
entry = payload.get("entries", {}).get(wrapper_rel)
if not isinstance(entry, dict):
    raise SystemExit(f"expected registry entry for {wrapper_rel}")
if entry.get("interpreter") != "bash":
    raise SystemExit(f"expected bash interpreter for {wrapper_rel}")
if entry.get("target") != "scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh":
    raise SystemExit(f"expected shared manifest migration dispatcher target for {wrapper_rel}")
if entry.get("args_prefix") != ["--group", expected_group]:
    raise SystemExit(
        f"expected args_prefix ['--group', {expected_group!r}] for {wrapper_rel}, got {entry.get('args_prefix')!r}"
    )
if entry.get("passthrough") is not True:
    raise SystemExit(f"expected passthrough=true for {wrapper_rel}")
PY
done

echo "Kolme manifest-migration contract dispatcher wrapper matrix tests passed."
