import {
  ALLOWED_MESSAGE_TYPES,
  CANONICAL_ENCRYPTION_ALGORITHM,
  CANONICAL_MESSAGE_ENVELOPE_TYPE,
  CANONICAL_PROOF_PURPOSE,
  type CanonicalMessageEnvelope,
  type CreateCanonicalEnvelopeInput,
} from "./types.ts";

const DID_PATTERN = /^kamn:did:agent:[A-Za-z0-9._:-]+$/;

export class SchemaError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "SchemaError";
  }
}

export function createCanonicalMessageEnvelope(
  input: CreateCanonicalEnvelopeInput,
): CanonicalMessageEnvelope {
  const created = input.created ?? timestampFromNonce(input.nonce, 0);
  const expires = input.expires ?? timestampFromNonce(input.nonce, 60);

  return {
    envelope: {
      id: input.id,
      typeName: CANONICAL_MESSAGE_ENVELOPE_TYPE,
      from: input.from,
      to: [...input.to],
      created,
      expires,
      threadId: input.threadId,
      parentId: input.parentId,
      nonce: input.nonce,
    },
    header: {
      messageType: input.messageType,
      priority: input.priority ?? "normal",
      contentType: input.contentType ?? "application/json",
      encryption: {
        algorithm: CANONICAL_ENCRYPTION_ALGORITHM,
        recipientKeys: [...input.recipientKeys],
      },
    },
    body: { ...input.body },
    attachments: [...(input.attachments ?? [])],
    proof: {
      typeName: "Ed25519Signature2020",
      created,
      verificationMethod: `${input.from}#key-1`,
      proofPurpose: CANONICAL_PROOF_PURPOSE,
      proofValue: input.proofValue ?? "proof-placeholder",
    },
  };
}

export function validateCanonicalMessageEnvelope(
  envelope: CanonicalMessageEnvelope,
): void {
  requireNonEmpty("envelope.id", envelope.envelope.id);
  if (envelope.envelope.typeName !== CANONICAL_MESSAGE_ENVELOPE_TYPE) {
    throw new SchemaError(
      `envelope.type_name must be ${CANONICAL_MESSAGE_ENVELOPE_TYPE}`,
    );
  }

  if (!DID_PATTERN.test(envelope.envelope.from)) {
    throw new SchemaError(`invalid sender did: ${envelope.envelope.from}`);
  }
  if (envelope.envelope.to.length === 0) {
    throw new SchemaError("envelope.to must not be empty");
  }
  for (const recipient of envelope.envelope.to) {
    if (!DID_PATTERN.test(recipient)) {
      throw new SchemaError(`invalid recipient did: ${recipient}`);
    }
  }

  requireNonEmpty("envelope.created", envelope.envelope.created);
  requireNonEmpty("envelope.expires", envelope.envelope.expires);
  const createdMs = Date.parse(envelope.envelope.created);
  const expiresMs = Date.parse(envelope.envelope.expires);
  if (Number.isNaN(createdMs) || Number.isNaN(expiresMs)) {
    throw new SchemaError("envelope.created and envelope.expires must be valid ISO timestamps");
  }
  if (expiresMs <= createdMs) {
    throw new SchemaError("envelope.expires must be strictly after envelope.created");
  }

  if (!Number.isInteger(envelope.envelope.nonce) || envelope.envelope.nonce <= 0) {
    throw new SchemaError("envelope.nonce must be a positive integer");
  }

  requireNonEmpty("header.message_type", envelope.header.messageType);
  if (!ALLOWED_MESSAGE_TYPES.includes(envelope.header.messageType as never)) {
    throw new SchemaError(
      "header.message_type must be one of the canonical values",
    );
  }
  requireNonEmpty("header.priority", envelope.header.priority);
  requireNonEmpty("header.content_type", envelope.header.contentType);

  if (envelope.header.encryption.algorithm !== CANONICAL_ENCRYPTION_ALGORITHM) {
    throw new SchemaError(
      `header.encryption.algorithm must be ${CANONICAL_ENCRYPTION_ALGORITHM}`,
    );
  }
  if (envelope.header.encryption.recipientKeys.length === 0) {
    throw new SchemaError("header.encryption.recipient_keys must not be empty");
  }
  for (const keyRef of envelope.header.encryption.recipientKeys) {
    if (!keyRef.trim()) {
      throw new SchemaError("header.encryption.recipient_keys[] must not be empty");
    }
  }

  const entries = Object.entries(envelope.body);
  if (entries.length === 0) {
    throw new SchemaError("body must not be empty");
  }
  for (const [key, value] of entries) {
    if (!key.trim() || !value.trim()) {
      throw new SchemaError("body entries must have non-empty key/value");
    }
  }

  for (const attachment of envelope.attachments) {
    if (!attachment.id.trim()) {
      throw new SchemaError("attachment.id must not be empty");
    }
    if (!attachment.mediaType.trim()) {
      throw new SchemaError("attachment.media_type must not be empty");
    }
    if (!attachment.uri.trim()) {
      throw new SchemaError("attachment.uri must not be empty");
    }
  }

  requireNonEmpty("proof.type_name", envelope.proof.typeName);
  requireNonEmpty("proof.created", envelope.proof.created);
  requireNonEmpty("proof.verification_method", envelope.proof.verificationMethod);
  if (envelope.proof.proofPurpose !== CANONICAL_PROOF_PURPOSE) {
    throw new SchemaError(`proof.proof_purpose must be ${CANONICAL_PROOF_PURPOSE}`);
  }
  requireNonEmpty("proof.proof_value", envelope.proof.proofValue);

  const expectedPrefix = `${envelope.envelope.from}#`;
  if (!envelope.proof.verificationMethod.startsWith(expectedPrefix)) {
    throw new SchemaError(
      `proof.verification_method must start with ${expectedPrefix}`,
    );
  }
}

export function canonicalPayload(envelope: CanonicalMessageEnvelope): string {
  const recipients = [...envelope.envelope.to].sort();
  const recipientKeys = [...envelope.header.encryption.recipientKeys].sort();
  const attachments = [...envelope.attachments].sort((a, b) =>
    a.id.localeCompare(b.id),
  );
  const bodyEntries = Object.entries(envelope.body).sort((a, b) =>
    a[0].localeCompare(b[0]),
  );

  const parts: string[] = [];
  parts.push("envelope");
  parts.push(envelope.envelope.id);
  parts.push(envelope.envelope.typeName);
  parts.push(envelope.envelope.from);
  parts.push(recipients.join(","));
  parts.push(envelope.envelope.created);
  parts.push(envelope.envelope.expires);
  parts.push(envelope.envelope.threadId ?? "");
  parts.push(envelope.envelope.parentId ?? "");
  parts.push(String(envelope.envelope.nonce));

  parts.push("header");
  parts.push(envelope.header.messageType);
  parts.push(envelope.header.priority);
  parts.push(envelope.header.contentType);
  parts.push(envelope.header.encryption.algorithm);
  parts.push(recipientKeys.join(","));

  parts.push("body");
  let bodyPayload = "";
  for (const [key, value] of bodyEntries) {
    bodyPayload += `${key}=${value};`;
  }
  parts.push(bodyPayload);

  parts.push("attachments");
  let attachmentPayload = "";
  for (const attachment of attachments) {
    attachmentPayload += `${attachment.id}:${attachment.mediaType}:${attachment.uri};`;
  }
  parts.push(attachmentPayload);

  parts.push("proof");
  parts.push(envelope.proof.typeName);
  parts.push(envelope.proof.created);
  parts.push(envelope.proof.verificationMethod);
  parts.push(envelope.proof.proofPurpose);
  parts.push(envelope.proof.proofValue);

  return parts.join("|");
}

function requireNonEmpty(field: string, value: string): void {
  if (!value.trim()) {
    throw new SchemaError(`${field} must not be empty`);
  }
}

function timestampFromNonce(nonce: number, offsetSeconds: number): string {
  const baseMs = Date.UTC(2026, 0, 1, 0, 0, 0);
  const nonceSeconds = Number.isFinite(nonce) ? Math.max(0, nonce) : 0;
  return new Date(baseMs + (nonceSeconds + offsetSeconds) * 1000).toISOString();
}
