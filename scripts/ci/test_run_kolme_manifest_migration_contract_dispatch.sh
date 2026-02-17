#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/ci/run_kolme_manifest_migration_contract_dispatch.sh"
CONFIG_FILE="$ROOT_DIR/fixtures/ci/kolme_manifest_migration_contract_groups.json"

if [ ! -x "$DISPATCHER" ]; then
  echo "expected dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [ ! -f "$CONFIG_FILE" ]; then
  echo "expected migration config file to exist: $CONFIG_FILE" >&2
  exit 1
fi

if ! output="$(bash "$DISPATCHER" --group tranche1)"; then
  echo "expected dispatcher tranche1 execution to pass" >&2
  exit 1
fi
if ! grep -Fq "Kolme tranche-1 manifest migration contract lane checks passed." <<<"$output"; then
  echo "expected dispatcher success output for tranche1 group" >&2
  exit 1
fi

if bash "$DISPATCHER" --group unknown_group >/tmp/kolme-migration-dispatch-unknown.out 2>&1; then
  echo "expected dispatcher to fail for unknown group key" >&2
  exit 1
fi
if ! grep -Fq "unknown migration group key: unknown_group" /tmp/kolme-migration-dispatch-unknown.out; then
  echo "expected unknown-group failure message from dispatcher" >&2
  exit 1
fi

if bash "$DISPATCHER" >/tmp/kolme-migration-dispatch-missing-group.out 2>&1; then
  echo "expected dispatcher to fail when group is missing" >&2
  exit 1
fi
if ! grep -Fq "expected --group to be provided" /tmp/kolme-migration-dispatch-missing-group.out; then
  echo "expected missing-group failure message from dispatcher" >&2
  exit 1
fi

temp_config="$(mktemp)"
trap 'rm -f "$temp_config" /tmp/kolme-migration-dispatch-unknown.out /tmp/kolme-migration-dispatch-missing-group.out /tmp/kolme-migration-dispatch-schema.out' EXIT
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$temp_config" <<'EOF'
{"schema_version":"invalid-schema","groups":{}}
EOF

if bash "$DISPATCHER" --group tranche1 --config-file "$temp_config" >/tmp/kolme-migration-dispatch-schema.out 2>&1; then
  echo "expected dispatcher to fail for invalid config schema" >&2
  exit 1
fi
if ! grep -Fq "unexpected Kolme migration config schema version" /tmp/kolme-migration-dispatch-schema.out; then
  echo "expected schema validation failure message from dispatcher" >&2
  exit 1
fi

echo "Kolme manifest-migration dispatcher contract lane tests passed."
