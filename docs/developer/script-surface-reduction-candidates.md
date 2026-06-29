# Script Surface Reduction Candidates

script_surface_short_wrapper_schema_version=kamn.docs.script-surface-short-wrapper-candidates.v1
script_surface_short_wrapper_generated_on=2026-06-26
script_surface_short_wrapper_shell_threshold_max_loc=25
script_surface_short_wrapper_python_threshold_max_loc=40
script_surface_short_wrapper_category_count=24
script_surface_short_wrapper_total_candidates=62
script_surface_short_wrapper_priority_categories_csv=ci,sdk,runtime,kolme,bridge,framework

This candidate matrix prioritizes short-wrapper consolidation opportunities by
category using deterministic thresholds:

- Shell short-wrapper candidate: `*.sh` file with LOC `<= 25`
- Python short-wrapper candidate: `*.py` file with LOC `<= 40`

## Candidate Matrix

| Category | Total scripts | Short-wrapper candidates | Candidate ratio |
| --- | --- | --- | --- |
| `scripts/ci` | `215` | `20` | `9.30%` |
| `scripts/sdk` | `80` | `16` | `20.00%` |
| `scripts/runtime` | `232` | `5` | `2.16%` |
| `scripts/kolme` | `217` | `5` | `2.30%` |
| `scripts/bridge` | `42` | `3` | `7.14%` |
| `scripts/framework` | `36` | `3` | `8.33%` |
| `scripts/message` | `26` | `2` | `7.69%` |
| `scripts/task` | `11` | `2` | `18.18%` |
| `scripts/lib` | `8` | `2` | `25.00%` |
| `scripts/deploy` | `40` | `1` | `2.50%` |
| `scripts/signer` | `17` | `1` | `5.88%` |
| `scripts/channel` | `9` | `1` | `11.11%` |
| `scripts/guard` | `3` | `1` | `33.33%` |
| `scripts/did` | `30` | `0` | `0.00%` |
| `scripts/governance` | `28` | `0` | `0.00%` |
| `scripts/reputation` | `27` | `0` | `0.00%` |
| `scripts/compliance` | `22` | `0` | `0.00%` |
| `scripts/dashboard` | `12` | `0` | `0.00%` |
| `scripts/cutover` | `11` | `0` | `0.00%` |
| `scripts/canary` | `10` | `0` | `0.00%` |
| `scripts/frontend` | `9` | `0` | `0.00%` |
| `scripts/escrow` | `7` | `0` | `0.00%` |
| `scripts/token` | `6` | `0` | `0.00%` |
| `scripts/treasury` | `4` | `0` | `0.00%` |

## Regeneration Commands

```bash
# Combined category totals + short-wrapper candidates by deterministic thresholds:
{ find scripts -type f -name '*.sh' -exec wc -l {} + \
    | awk 'NF==2 && $2!="total"{split($2,p,"/"); c=p[2]; total[c]++; if($1<=25) short[c]++} END{for(k in total) print k"|"total[k]"|"short[k]+0}'; \
  find scripts -type f -name '*.py' -exec wc -l {} + \
    | awk 'NF==2 && $2!="total"{split($2,p,"/"); c=p[2]; total[c]++; if($1<=40) short[c]++} END{for(k in total) print k"|"total[k]"|"short[k]+0}'; } \
  | awk -F'|' '{cat=$1; total[cat]+=$2; short[cat]+=$3} END{for(c in total) printf "%s|%d|%d|%.2f\n", c, total[c], short[c], (short[c]/total[c])*100}' \
  | sort -t'|' -k3,3nr -k2,2nr
```
