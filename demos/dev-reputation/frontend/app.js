/* global ethers, keccak256 */
const state = {
  leaderboard: [],
  selected: null,
};

function scoreClass(value) {
  if (value >= 75) return "score-good";
  if (value >= 50) return "score-mid";
  return "score-low";
}

function tierBadge(tier) {
  return `<span class="badge">${tier}</span>`;
}

function truncCid(cid = "") {
  if (cid.length < 16) return cid;
  return `${cid.slice(0, 8)}...${cid.slice(-6)}`;
}

function initials(handle = "") {
  return handle.slice(0, 2).toUpperCase();
}

function renderLeaderboard() {
  const body = document.getElementById("leaderboard-body");
  body.innerHTML = "";

  const sorted = [...state.leaderboard].sort((a, b) => b.score - a.score);
  sorted.forEach((row, idx) => {
    const tr = document.createElement("tr");
    tr.innerHTML = `
      <td>${idx + 1}</td>
      <td>${initials(row.github_handle)}</td>
      <td><a href="#" data-profile="${row.github_handle}">${row.github_handle}</a></td>
      <td class="${scoreClass(row.score)}">${row.score}</td>
      <td>${tierBadge(row.tier)}</td>
      <td>${row.pr_analyses?.filter((p) => p.weight_multiplier > 0).length ?? 0}</td>
      <td>${row.adjusted_stats?.inflation_removed_pct ?? 0}%</td>
      <td>${new Date(row.updated_at).toLocaleString()}</td>
      <td><a target="_blank" href="${window.IPC_CONFIG.basinUrl}/cid/${row.evidence_cid}">${truncCid(row.evidence_cid)}</a></td>
      <td><button data-verify="${row.github_handle}">Verify</button><div id="verify-${row.github_handle}"></div></td>
    `;
    body.appendChild(tr);
  });

  body.querySelectorAll("a[data-profile]").forEach((el) => {
    el.addEventListener("click", (ev) => {
      ev.preventDefault();
      openProfile(el.dataset.profile);
    });
  });
  body.querySelectorAll("button[data-verify]").forEach((el) => {
    el.addEventListener("click", () => verifyRow(el.dataset.verify));
  });
}

function renderProfile() {
  const row = state.selected;
  if (!row) return;

  document.getElementById("profile-header").innerHTML = `
    <h2>${row.github_handle} <span class="badge">${row.tier.toUpperCase()}</span></h2>
    <p>Score: <strong class="${scoreClass(row.score)}">${row.score}</strong> | period: ${row.score_breakdown?.period || "90-day window"} | block: ${row.on_chain_submission?.block_height || "n/a"} | <span class="badge">F3 finality demo</span></p>
  `;

  const summary = [
    ["Effective PRs", row.pr_analyses?.filter((p) => p.weight_multiplier > 0).length ?? 0],
    ["Weighted Commits", row.adjusted_stats?.weighted_commits ?? 0],
    ["Inflation Removed", `${row.adjusted_stats?.inflation_removed_pct ?? 0}%`],
    ["Evidence CID", truncCid(row.evidence_cid)],
  ];
  document.getElementById("summary-row").innerHTML = summary
    .map(([k, v]) => `<div class="summary-card"><div class="muted">${k}</div><div>${v}</div></div>`)
    .join("");

  const dimensions = row.score_breakdown?.dimension_contributions || {};
  const labels = Object.keys(dimensions);
  document.getElementById("breakdown-bars").innerHTML = labels
    .map((key) => {
      const score = Math.max(0, Math.min(100, Math.round((dimensions[key] / 0.28) * 0.28)));
      const cls = score >= 75 ? "#2ecc71" : score >= 50 ? "#f5b041" : "#ff5e5e";
      return `
        <div class="bar-row">
          <div>${key}</div>
          <div class="bar"><div class="bar-fill" style="width:${score}%;background:${cls};"></div></div>
          <div>${score}/100</div>
        </div>
      `;
    })
    .join("");

  const prBody = document.getElementById("pr-table-body");
  prBody.innerHTML = "";
  (row.pr_analyses || []).forEach((pr) => {
    const tr = document.createElement("tr");
    const warning = pr.weight_multiplier <= 0.1 ? ' style="background:rgba(255,94,94,0.1)"' : "";
    tr.innerHTML = `
      <td${warning}><a target="_blank" href="${pr.pr_url}">#${pr.pr_number} ${pr.pr_title}</a></td>
      <td${warning}>+${pr.raw_additions}/-${pr.raw_deletions}</td>
      <td${warning}>${pr.pr_score}</td>
      <td${warning}>${pr.weight_multiplier.toFixed(2)}</td>
      <td${warning}>${pr.verdict}</td>
    `;
    prBody.appendChild(tr);
  });

  const flagsWrap = document.getElementById("flags-wrap");
  const flagsList = document.getElementById("flags-list");
  if (row.gaming_flags && row.gaming_flags.length) {
    flagsWrap.classList.remove("hidden");
    flagsList.innerHTML = row.gaming_flags
      .map((f) => `<li><strong>${f.pattern}</strong>: ${f.description} (weight=${f.weight_applied})</li>`)
      .join("");
  } else {
    flagsWrap.classList.add("hidden");
    flagsList.innerHTML = "";
  }

  document.getElementById("audit").innerHTML = `
    <div>Document hash: <code>${row.document_hash}</code></div>
    <div>Agent address: <code>${row.agent_address}</code></div>
    <div>Signature: <code>${row.agent_signature}</code></div>
    <div>Basin CID: <code>${row.evidence_cid}</code></div>
    <button id="verify-onchain-btn">Verify on-chain</button>
    <div id="verify-onchain-result"></div>
  `;

  document.getElementById("verify-onchain-btn").addEventListener("click", async () => {
    const out = document.getElementById("verify-onchain-result");
    out.textContent = "Checking on-chain record...";
    const ok = await verifyOnChain(row);
    out.textContent = ok ? "On-chain verification passed" : "On-chain verification could not be confirmed";
  });
}

async function rpcCall(method, params) {
  const response = await fetch(window.IPC_CONFIG.rpcUrl, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const body = await response.json();
  if (body.error) throw new Error(body.error.message || "RPC error");
  return body.result;
}

async function fetchBasinDoc(cid) {
  const url = `${window.IPC_CONFIG.basinUrl}/api/v1/buckets/${encodeURIComponent(
    window.IPC_CONFIG.basinBucket || "default"
  )}/objects/${encodeURIComponent(cid)}`;
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Basin fetch failed: ${response.status}`);
  }
  return response.text();
}

async function verifyRow(handle) {
  const row = state.leaderboard.find((r) => r.github_handle === handle);
  const target = document.getElementById(`verify-${handle}`);
  if (!row || !target) return;

  target.innerHTML = `<div><span class="spinner"></span> running checks...</div>`;
  try {
    const docText = await fetchBasinDoc(row.evidence_cid);
    const computedHash = `0x${keccak256(docText)}`;
    const contentOk = computedHash.toLowerCase() === String(row.document_hash).toLowerCase();

    const recovered = ethers.utils.verifyMessage(ethers.utils.arrayify(row.document_hash), row.agent_signature);
    const agentOk = recovered.toLowerCase() === String(row.agent_address).toLowerCase();

    let blockOk = false;
    let blockInfo = "n/a";
    try {
      const tipset = await rpcCall("Filecoin.ChainGetTipSetByHeight", [
        row.on_chain_submission?.block_height || 0,
        null,
      ]);
      const ts = Number(tipset?.Blocks?.[0]?.Timestamp || 0);
      const expected = Math.floor(new Date(row.updated_at).getTime() / 1000);
      blockOk = Math.abs(ts - expected) <= 60;
      blockInfo = `computed=${ts} expected=${expected}`;
    } catch (_e) {
      blockInfo = "unavailable";
    }

    target.innerHTML = `
      <div>integrity: ${contentOk ? "PASS" : "FAIL"} (computed=${computedHash}, expected=${row.document_hash})</div>
      <div>agent identity: ${agentOk ? "PASS" : "FAIL"} (computed=${recovered}, expected=${row.agent_address})</div>
      <div>block timestamp: ${blockOk ? "PASS" : "FAIL"} (${blockInfo})</div>
    `;
  } catch (error) {
    target.textContent = `Verification error: ${error.message}`;
  }
}

async function verifyOnChain(row) {
  try {
    const result = await rpcCall("IPC.ReputationGetScore", [window.IPC_CONFIG.actorAddress, row.wallet_address]);
    if (!result) return false;
    return String(result.evidence_cid || "").toLowerCase() === String(row.evidence_cid || "").toLowerCase();
  } catch (_e) {
    return false;
  }
}

function openProfile(handle) {
  state.selected = state.leaderboard.find((row) => row.github_handle === handle) || null;
  renderProfile();
  document.getElementById("leaderboard-view").classList.add("hidden");
  document.getElementById("profile-view").classList.remove("hidden");
}

function closeProfile() {
  state.selected = null;
  document.getElementById("profile-view").classList.add("hidden");
  document.getElementById("leaderboard-view").classList.remove("hidden");
}

async function pollJob(jobId) {
  const statusEl = document.getElementById("job-status");
  for (;;) {
    const response = await fetch(`${window.IPC_CONFIG.agentUrl}/job/${jobId}`);
    const job = await response.json();
    statusEl.textContent = `${job.status} - ${job.progress?.step || "waiting"} (${job.progress?.percentage || 0}%)`;
    if (job.status === "complete") {
      const scoreResponse = await fetch(`${window.IPC_CONFIG.agentUrl}/score/${state.pendingHandle}`);
      const result = await scoreResponse.json();
      state.leaderboard = [result, ...state.leaderboard.filter((r) => r.github_handle !== result.github_handle)];
      renderLeaderboard();
      return;
    }
    if (job.status === "error") {
      statusEl.textContent = `error: ${job.error}`;
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
}

async function scoreHandle() {
  const github_handle = document.getElementById("handle-input").value.trim();
  const wallet_address = document.getElementById("wallet-input").value.trim();
  if (!github_handle || !wallet_address) {
    alert("Provide GitHub handle and wallet address");
    return;
  }
  state.pendingHandle = github_handle;
  const response = await fetch(`${window.IPC_CONFIG.agentUrl}/score`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ github_handle, wallet_address }),
  });
  const body = await response.json();
  if (!response.ok) {
    document.getElementById("job-status").textContent = `error: ${body.error || "request failed"}`;
    return;
  }
  pollJob(body.job_id);
}

document.getElementById("score-btn").addEventListener("click", scoreHandle);
document.getElementById("back-btn").addEventListener("click", closeProfile);
renderLeaderboard();
