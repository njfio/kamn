export {
  canonicalPayload,
  createCanonicalMessageEnvelope,
  SchemaError,
  validateCanonicalMessageEnvelope,
} from "./message_envelope.ts";

export {
  ALLOWED_MESSAGE_TYPES,
  CANONICAL_ENCRYPTION_ALGORITHM,
  CANONICAL_MESSAGE_ENVELOPE_TYPE,
  CANONICAL_PROOF_PURPOSE,
} from "./types.ts";

export type {
  AttachmentRef,
  CanonicalMessageEnvelope,
  CanonicalMessageType,
  CreateCanonicalEnvelopeInput,
  EnvelopeEncryption,
  EnvelopeHeader,
  EnvelopeMetadata,
  EnvelopeProof,
} from "./types.ts";
