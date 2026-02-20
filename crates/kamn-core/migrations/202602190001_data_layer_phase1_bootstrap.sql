-- KAMN Data Layer Phase 1 Bootstrap
-- Issue: #5255
-- Milestone: R27.45

BEGIN;

CREATE TABLE IF NOT EXISTS merkle_batches (
    batch_id UUID PRIMARY KEY,
    root_hash TEXT NOT NULL,
    leaf_count INTEGER NOT NULL CHECK (leaf_count > 0),
    status TEXT NOT NULL,
    kolme_tx_hash TEXT,
    kolme_block_height BIGINT,
    scheduled_at TIMESTAMPTZ NOT NULL,
    submitted_at TIMESTAMPTZ,
    confirmed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS did_registry (
    did TEXT PRIMARY KEY,
    owner_did TEXT NOT NULL,
    document_json JSONB NOT NULL DEFAULT '{}'::JSONB,
    capabilities JSONB NOT NULL DEFAULT '[]'::JSONB,
    signing_key_fingerprint TEXT NOT NULL,
    encryption_key_fingerprint TEXT NOT NULL,
    delegation_chain_json JSONB NOT NULL DEFAULT '[]'::JSONB,
    state TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS escrows (
    escrow_id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    payer_did TEXT NOT NULL,
    payee_did TEXT NOT NULL,
    auditor_did TEXT,
    state TEXT NOT NULL,
    release_threshold INTEGER NOT NULL DEFAULT 1,
    amount_atomic BIGINT NOT NULL CHECK (amount_atomic >= 0),
    currency_code TEXT NOT NULL,
    evidence_json JSONB NOT NULL DEFAULT '[]'::JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS key_rotation_log (
    rotation_id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    key_fingerprint TEXT NOT NULL,
    key_usage TEXT NOT NULL,
    previous_key_fingerprint TEXT,
    reason_code TEXT NOT NULL,
    rotated_at TIMESTAMPTZ NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS access_log (
    access_id UUID PRIMARY KEY,
    requester_did TEXT NOT NULL,
    owner_did TEXT NOT NULL,
    action TEXT NOT NULL,
    resource_kind TEXT NOT NULL,
    resource_id TEXT NOT NULL,
    decision TEXT NOT NULL,
    reason_code TEXT NOT NULL,
    request_metadata JSONB NOT NULL DEFAULT '{}'::JSONB,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS messages (
    message_id UUID PRIMARY KEY,
    owner_did TEXT NOT NULL,
    sender_did TEXT NOT NULL,
    recipient_did TEXT NOT NULL,
    escrow_id UUID REFERENCES escrows(escrow_id),
    merkle_batch_id UUID REFERENCES merkle_batches(batch_id),
    merkle_leaf_index INTEGER,
    envelope_ciphertext BYTEA NOT NULL,
    wrapped_keys JSONB NOT NULL DEFAULT '[]'::JSONB,
    envelope_nonce BIGINT NOT NULL,
    compression_codec TEXT NOT NULL,
    compression_dictionary_id TEXT,
    content_hash_sha256 TEXT NOT NULL,
    hash_chain_prev TEXT NOT NULL,
    hash_chain_curr TEXT NOT NULL,
    blind_indexes JSONB NOT NULL DEFAULT '{}'::JSONB,
    metadata_json JSONB NOT NULL DEFAULT '{}'::JSONB,
    retention_class TEXT NOT NULL,
    legal_hold BOOLEAN NOT NULL DEFAULT FALSE,
    shredded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_messages_owner_created_at
    ON messages(owner_did, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_messages_non_shredded_created_at
    ON messages(created_at DESC)
    WHERE shredded_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_messages_blind_indexes_gin
    ON messages USING GIN (blind_indexes jsonb_path_ops);

CREATE INDEX IF NOT EXISTS idx_messages_sender_recipient_created_at
    ON messages(sender_did, recipient_did, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_merkle_batches_status_scheduled_at
    ON merkle_batches(status, scheduled_at);

CREATE INDEX IF NOT EXISTS idx_did_registry_owner_state
    ON did_registry(owner_did, state);

CREATE INDEX IF NOT EXISTS idx_escrows_owner_state
    ON escrows(owner_did, state);

CREATE INDEX IF NOT EXISTS idx_key_rotation_log_owner_rotated_at
    ON key_rotation_log(owner_did, rotated_at DESC);

CREATE INDEX IF NOT EXISTS idx_access_log_owner_recorded_at
    ON access_log(owner_did, recorded_at DESC);

-- KAMN_M2_RLS_MARKER:ENABLE_RLS_MESSAGES
ALTER TABLE messages ENABLE ROW LEVEL SECURITY;

-- KAMN_M2_RLS_MARKER:MESSAGES_OWNER_SELECT_POLICY_TEMPLATE
DROP POLICY IF EXISTS messages_owner_select ON messages;
CREATE POLICY messages_owner_select
    ON messages
    FOR SELECT
    USING (owner_did = current_setting('kamn.requester_did', TRUE));

-- KAMN_M3_INDEX_MARKER:BLIND_INDEX_GIN_READY
-- Blind-index JSONB storage and GIN index are provisioned for M3 lookup contracts.

-- KAMN_M8_RETENTION_MARKER:SHREDDED_AT_PARTIAL_INDEX_READY
-- `idx_messages_non_shredded_created_at` enforces retention/shredding query locality.

COMMIT;
