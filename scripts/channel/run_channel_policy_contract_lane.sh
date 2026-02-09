#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RETENTION_REDACTION_LANE="$ROOT_DIR/scripts/channel/run_channel_retention_redaction_contract_lane.sh"

cargo test -p kamn-core --lib channel_policies::tests:: >/dev/null
cargo test -p kamn-core --test channel_permissions_retention >/dev/null
cargo test -p kamn-core --test channel_permissions_retention_docs >/dev/null
cargo test -p kamn-core --test channel_models_and_permissions_docs >/dev/null
bash "$RETENTION_REDACTION_LANE" >/dev/null

echo "channel policy contract lane tests passed."
