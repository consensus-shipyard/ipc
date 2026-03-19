const axios = require("axios");

async function writeEvidenceToBasin(document, { basinApiUrl, bucket }) {
  const url = `${basinApiUrl.replace(/\/$/, "")}/api/v1/buckets/${encodeURIComponent(bucket)}/objects`;
  const response = await axios.post(url, document, {
    headers: { "Content-Type": "application/json" },
    timeout: 30000,
  });
  const cid = response.data?.cid || response.data?.data?.cid || response.data?.result?.cid;
  if (!cid) {
    throw new Error("Basin response missing CID");
  }
  return cid;
}

async function fetchEvidenceFromBasin(cid, { basinApiUrl, bucket }) {
  const url = `${basinApiUrl.replace(/\/$/, "")}/api/v1/buckets/${encodeURIComponent(bucket)}/objects/${encodeURIComponent(cid)}`;
  const response = await axios.get(url, { timeout: 30000 });
  return response.data;
}

module.exports = {
  writeEvidenceToBasin,
  fetchEvidenceFromBasin,
};
