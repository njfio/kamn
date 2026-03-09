# Specs Index

specs_index_version=kamn.docs.specs-index.v2
specs_index_purpose=full top-level issue spec coverage and workflow orientation
specs_index_scope=top_level_issue_specs_only
specs_index_naming_pattern=specs/{issue}-{slug}.md
specs_index_status_taxonomy_csv=planned,active,completed,superseded
specs_index_coverage_authority=scripts/ci/check_specs_index_coverage.sh
specs_index_shards_csv=specs/index/6000-6499.md,specs/index/6500-6999.md

## Purpose

Use this index entrypoint to navigate the complete top-level issue-spec corpus without folding nested archival planning documents into the closure-ready contract.

## Coverage Rule

- Authoritative corpus: top-level `specs/*.md` issue specs, excluding `specs/INDEX.md`
- Coverage contract: every top-level issue spec appears in exactly one shard
- Verification command: `bash scripts/ci/check_specs_index_coverage.sh --output-json /tmp/specs-index-coverage.json`

## Shards

- [6000-6499](./index/6000-6499.md)
- [6500-6999](./index/6500-6999.md)
