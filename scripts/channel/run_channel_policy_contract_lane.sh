#!/usr/bin/env bash
set -euo pipefail

cargo test -p kamn-core --lib channel_policies::tests:: >/dev/null
cargo test -p kamn-core --test channel_permissions_retention >/dev/null
cargo test -p kamn-core --test channel_permissions_retention_docs >/dev/null
cargo test -p kamn-core --test channel_models_and_permissions_docs >/dev/null

echo "channel policy contract lane tests passed."
