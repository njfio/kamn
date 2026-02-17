# Plan: #4363 Signature-Decision Taxonomy Implementation

## Approach

1. Add signature-decision taxonomy constants and deterministic reason list.
2. Add helper to project observed signature-decision reasons from normalized checker reason codes.
3. Add output JSON/stdout markers:
   - `signature_decision_reason_taxonomy_version`
   - `signature_decision_reason_codes_csv`
   - `signature_decision_reason_codes_value`
4. Update ops configuration doc markers and validation commands.
5. Validate with targeted and full required gates.
