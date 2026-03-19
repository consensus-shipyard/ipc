const { ethers } = require("ethers");

function canonicalizeTopLevel(evidence) {
  const sortedKeys = Object.keys(evidence).sort();
  return JSON.stringify(evidence, sortedKeys);
}

function hashEvidence(evidence) {
  const canonical = canonicalizeTopLevel(evidence);
  return ethers.utils.keccak256(ethers.utils.toUtf8Bytes(canonical));
}

async function signEvidence(evidence, privateKeyNoPrefix) {
  const wallet = new ethers.Wallet(`0x${privateKeyNoPrefix.replace(/^0x/, "")}`);
  const documentHash = hashEvidence(evidence);
  const signature = await wallet.signMessage(ethers.utils.arrayify(documentHash));
  return {
    ...evidence,
    document_hash: documentHash,
    agent_signature: signature,
    agent_address: wallet.address,
  };
}

function verifyEvidence(documentHash, signature, expectedAddress) {
  const recovered = ethers.utils.verifyMessage(ethers.utils.arrayify(documentHash), signature);
  return recovered.toLowerCase() === expectedAddress.toLowerCase();
}

module.exports = {
  canonicalizeTopLevel,
  hashEvidence,
  signEvidence,
  verifyEvidence,
};
