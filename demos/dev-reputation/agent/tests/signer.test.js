const test = require("node:test");
const assert = require("node:assert/strict");
const { hashEvidence, signEvidence, verifyEvidence } = require("../src/signer");

const PRIVATE_KEY = "59c6995e998f97a5a0044966f094538c5f2f5d4d2f3ecf0f7d8ef6a5a9d2d8f7";

test("sign and verify roundtrip", async () => {
  const evidence = { schema_version: "1.0", developer: { github_handle: "alice" } };
  const signed = await signEvidence(evidence, PRIVATE_KEY);
  assert.equal(verifyEvidence(signed.document_hash, signed.agent_signature, signed.agent_address), true);
});

test("modified document hash fails original signature verification", async () => {
  const evidence = { schema_version: "1.0", score: 75 };
  const signed = await signEvidence(evidence, PRIVATE_KEY);
  const modifiedHash = hashEvidence({ ...evidence, score: 76 });
  assert.notEqual(modifiedHash, signed.document_hash);
  assert.equal(verifyEvidence(modifiedHash, signed.agent_signature, signed.agent_address), false);
});
