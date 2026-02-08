export const CANONICAL_MESSAGE_ENVELOPE_TYPE = "kamn:message:v1";
export const CANONICAL_ENCRYPTION_ALGORITHM = "X25519-XChaCha20-Poly1305";
export const CANONICAL_PROOF_PURPOSE = "authentication";

export const ALLOWED_MESSAGE_TYPES = [
  "Request",
  "Response",
  "Proposal",
  "Acceptance",
  "Rejection",
  "Delegation",
  "StatusUpdate",
  "PaymentOffer",
  "PaymentConfirm",
  "Heartbeat",
  "Revocation",
] as const;

export type CanonicalMessageType = (typeof ALLOWED_MESSAGE_TYPES)[number];

export interface EnvelopeMetadata {
  id: string;
  typeName: string;
  from: string;
  to: string[];
  created: string;
  expires: string;
  threadId?: string;
  parentId?: string;
  nonce: number;
}

export interface EnvelopeEncryption {
  algorithm: string;
  recipientKeys: string[];
}

export interface EnvelopeHeader {
  messageType: string;
  priority: string;
  contentType: string;
  encryption: EnvelopeEncryption;
}

export interface AttachmentRef {
  id: string;
  mediaType: string;
  uri: string;
}

export interface EnvelopeProof {
  typeName: string;
  created: string;
  verificationMethod: string;
  proofPurpose: string;
  proofValue: string;
}

export interface CanonicalMessageEnvelope {
  envelope: EnvelopeMetadata;
  header: EnvelopeHeader;
  body: Record<string, string>;
  attachments: AttachmentRef[];
  proof: EnvelopeProof;
}

export interface CreateCanonicalEnvelopeInput {
  id: string;
  from: string;
  to: string[];
  nonce: number;
  messageType: string;
  body: Record<string, string>;
  recipientKeys: string[];
  created?: string;
  expires?: string;
  priority?: string;
  contentType?: string;
  attachments?: AttachmentRef[];
  threadId?: string;
  parentId?: string;
  proofValue?: string;
}
