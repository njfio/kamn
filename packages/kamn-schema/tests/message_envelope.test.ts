import test from "node:test";
import assert from "node:assert/strict";

import {
  canonicalPayload,
  createCanonicalMessageEnvelope,
  SchemaError,
  validateCanonicalMessageEnvelope,
} from "../src/index.ts";

test("valid canonical envelope passes and payload is deterministic", () => {
  const envelope = createCanonicalMessageEnvelope({
    id: "msg-1",
    from: "kamn:did:agent:sender-1",
    to: ["kamn:did:agent:recipient-b", "kamn:did:agent:recipient-a"],
    nonce: 1,
    messageType: "Request",
    body: {
      zeta: "2",
      alpha: "1",
    },
    recipientKeys: [
      "kamn:did:agent:recipient-b#enc-1",
      "kamn:did:agent:recipient-a#enc-1",
    ],
    attachments: [
      { id: "b", mediaType: "text/plain", uri: "ipfs://b" },
      { id: "a", mediaType: "text/plain", uri: "ipfs://a" },
    ],
  });

  validateCanonicalMessageEnvelope(envelope);
  const payload = canonicalPayload(envelope);

  assert.match(payload, /recipient-a,kamn:did:agent:recipient-b/);
  assert.match(payload, /alpha=1;zeta=2;/);
  assert.match(payload, /a:text\/plain:ipfs:\/\/a;b:text\/plain:ipfs:\/\/b;/);
});

test("rejects invalid message type", () => {
  const envelope = createCanonicalMessageEnvelope({
    id: "msg-2",
    from: "kamn:did:agent:sender-1",
    to: ["kamn:did:agent:recipient-1"],
    nonce: 2,
    messageType: "UnknownType",
    body: { text: "hello" },
    recipientKeys: ["kamn:did:agent:recipient-1#enc-1"],
  });

  assert.throws(() => validateCanonicalMessageEnvelope(envelope), {
    name: "SchemaError",
    message: /header\.message_type must be one of the canonical values/,
  });
});

test("regression rejects nonce zero", () => {
  // Regression: #218
  const envelope = createCanonicalMessageEnvelope({
    id: "msg-3",
    from: "kamn:did:agent:sender-1",
    to: ["kamn:did:agent:recipient-1"],
    nonce: 0,
    messageType: "Request",
    body: { text: "hello" },
    recipientKeys: ["kamn:did:agent:recipient-1#enc-1"],
  });

  assert.throws(() => validateCanonicalMessageEnvelope(envelope), SchemaError);
});
