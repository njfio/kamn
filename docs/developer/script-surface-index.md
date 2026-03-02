# Script Surface Index

script_surface_inventory_schema_version=kamn.docs.script-surface-index.v1
script_surface_inventory_generated_on=2026-03-02
script_surface_inventory_total_sh_files=746
script_surface_inventory_total_py_files=333
script_surface_inventory_total_files=1079
script_surface_inventory_category_count=24
script_surface_inventory_primary_categories_csv=runtime,kolme,ci,sdk,bridge,deploy,framework,did

This index is the canonical inventory baseline for `scripts/` shell/python surface.

## Inventory Totals

| File type | Count |
| --- | --- |
| `*.sh` | `746` |
| `*.py` | `333` |
| Combined (`*.sh` + `*.py`) | `1079` |

## Category Inventory

| Category | `.sh` | `.py` | Total |
| --- | --- | --- | --- |
| `scripts/runtime` | `180` | `49` | `229` |
| `scripts/kolme` | `117` | `100` | `217` |
| `scripts/ci` | `145` | `50` | `195` |
| `scripts/sdk` | `57` | `23` | `80` |
| `scripts/bridge` | `34` | `8` | `42` |
| `scripts/deploy` | `31` | `9` | `40` |
| `scripts/framework` | `19` | `17` | `36` |
| `scripts/did` | `21` | `9` | `30` |
| `scripts/governance` | `16` | `12` | `28` |
| `scripts/reputation` | `20` | `7` | `27` |
| `scripts/message` | `19` | `7` | `26` |
| `scripts/compliance` | `13` | `9` | `22` |
| `scripts/signer` | `13` | `4` | `17` |
| `scripts/dashboard` | `6` | `6` | `12` |
| `scripts/task` | `9` | `2` | `11` |
| `scripts/cutover` | `8` | `3` | `11` |
| `scripts/canary` | `6` | `4` | `10` |
| `scripts/frontend` | `6` | `3` | `9` |
| `scripts/channel` | `6` | `3` | `9` |
| `scripts/lib` | `7` | `1` | `8` |
| `scripts/escrow` | `5` | `2` | `7` |
| `scripts/token` | `4` | `2` | `6` |
| `scripts/treasury` | `2` | `2` | `4` |
| `scripts/guard` | `2` | `1` | `3` |

## Regeneration Commands

```bash
# Global totals
find scripts -type f -name '*.sh' | wc -l
find scripts -type f -name '*.py' | wc -l
find scripts -type f \( -name '*.sh' -o -name '*.py' \) | wc -l

# Per-category totals
find scripts -type f \( -name '*.sh' -o -name '*.py' \) \
  | awk -F/ '{count[$2]++} END {for (k in count) print k"|"count[k]}' \
  | sort

# Per-category extension split
find scripts -type f \( -name '*.sh' -o -name '*.py' \) \
  | awk -F/ '{cat=$2; ext=$NF; sub(/^.*\./,"",ext); key=cat"|"ext; counts[key]++} END {for (k in counts) print k"|"counts[k]}' \
  | sort
```
