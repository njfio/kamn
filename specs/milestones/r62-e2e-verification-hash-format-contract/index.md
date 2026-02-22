# Milestone R62 - E2E Verification Hash Format Contract

- Milestone: `R62 E2E Verification Hash Format Contract`
- Epic: #5654
- Completed issue(s): None
- Active issue(s): #5655
- Scope: enforce deterministic `sha256:` hash-format validation for evidence verification hash markers in verify flows.

## Delivery Slices
1. Verify-command rejection when `_verification.evidence_hash` or `_verification.kolme_anchor.tx_hash` lacks required `sha256:` prefix format, with deterministic diagnostics. (In Progress via #5655)
