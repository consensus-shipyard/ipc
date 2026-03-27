require("dotenv").config();

const express = require("express");
const axios = require("axios");
const crypto = require("crypto");
const { GitHubClient } = require("./github");
const { analysePR } = require("./analyser");
const { applyAntiCheat } = require("./anticheat");
const { signEvidence } = require("./signer");
const { writeEvidenceToBasin } = require("./basin");

const VERSION = "1.0.0";

const jobStore = new Map();
const latestScores = new Map();

function validateRequiredEnv() {
  const required = ["ANTHROPIC_API_KEY", "AGENT_PRIVATE_KEY", "BASIN_API_URL", "BASIN_BUCKET", "IPC_RPC_URL"];
  const missing = required.filter((key) => !String(process.env[key] || "").trim());
  if (missing.length) {
    throw new Error(`Missing required environment variables: ${missing.join(", ")}`);
  }
}

function tierForScore(score) {
  if (score >= 90) return { code: "T1", label: "principal" };
  if (score >= 75) return { code: "T2", label: "senior" };
  if (score >= 55) return { code: "T3", label: "mid" };
  if (score >= 35) return { code: "T4", label: "junior" };
  return { code: "T5", label: "early-career" };
}

function calculateConsistencyScore(commits, sinceIso) {
  const since = new Date(sinceIso).getTime();
  const weeks = new Map();
  for (let i = 0; i < 13; i += 1) {
    weeks.set(i, 0);
  }
  for (const commit of commits) {
    const ts = new Date(commit.commit?.author?.date || commit.commit?.committer?.date || Date.now()).getTime();
    const week = Math.max(0, Math.min(12, Math.floor((ts - since) / (7 * 24 * 60 * 60 * 1000))));
    weeks.set(week, (weeks.get(week) || 0) + 1);
  }
  const values = [...weeks.values()];
  const mean = values.reduce((a, b) => a + b, 0) / values.length || 0;
  if (mean === 0) {
    return 0;
  }
  const variance = values.reduce((sum, x) => sum + (x - mean) ** 2, 0) / values.length;
  const stddev = Math.sqrt(variance);
  const cv = stddev / mean;
  const normalized = Math.max(0, Math.min(1, 1 - cv / 2.0));
  return Math.round(normalized * 100);
}

function aggregateScore(prAnalyses, commits, sinceIso) {
  const weightedNumerator = prAnalyses.reduce(
    (sum, pr) => sum + Number(pr.pr_score || 0) * Number(pr.weight_multiplier || 0),
    0
  );
  const weightedDenominator = prAnalyses.reduce((sum, pr) => sum + Number(pr.weight_multiplier || 0), 0);
  const weightedPRScore = weightedDenominator > 0 ? weightedNumerator / weightedDenominator : 0;

  const consistencyScore = calculateConsistencyScore(commits, sinceIso);
  const overallScore = Math.round(weightedPRScore * 0.82 + consistencyScore * 0.18);
  const tier = tierForScore(overallScore);

  const avg = (key) =>
    prAnalyses.length
      ? prAnalyses.reduce((sum, p) => sum + Number(p.dimensions?.[key] || 0), 0) / prAnalyses.length
      : 0;
  const dimensionContributions = {
    problem_solving_depth: Number((avg("problem_solving_depth") * 0.28).toFixed(2)),
    code_quality: Number((avg("code_quality") * 0.23).toFixed(2)),
    review_responsiveness: Number((avg("review_responsiveness") * 0.14).toFixed(2)),
    consistency_over_time: Number((consistencyScore * 0.18).toFixed(2)),
    scope_appropriateness: Number((avg("scope_appropriateness") * 0.1).toFixed(2)),
    net_complexity_reduction: Number((avg("net_complexity_reduction") * 0.07).toFixed(2)),
  };

  return {
    weighted_pr_score: Number(weightedPRScore.toFixed(2)),
    consistency_score: consistencyScore,
    final_score: overallScore,
    tier: tier.label,
    tier_code: tier.code,
    dimension_contributions: dimensionContributions,
  };
}

function computePRScoreFromDimensions(dimensions) {
  const p = Number(dimensions.problem_solving_depth || 0);
  const c = Number(dimensions.code_quality || 0);
  const r = Number(dimensions.review_responsiveness || 50);
  const s = Number(dimensions.scope_appropriateness || 0);
  const n = Number(dimensions.net_complexity_reduction || 0);
  return Number((p * 0.3 + c * 0.25 + r * 0.15 + s * 0.15 + n * 0.15).toFixed(2));
}

async function submitOnChain(payload) {
  const rpcUrl = process.env.IPC_RPC_URL;
  const method = process.env.REPUTATION_SET_SCORE_METHOD;
  if (!rpcUrl || !method) {
    return { skipped: true, reason: "RPC method not configured" };
  }
  const response = await axios.post(
    rpcUrl,
    { jsonrpc: "2.0", id: 1, method, params: [payload] },
    { headers: { "Content-Type": "application/json" }, timeout: 30000 }
  );
  return { skipped: false, result: response.data?.result || null };
}

async function runPipeline({ github_handle, wallet_address, repo_filter, onProgress = () => {} }) {
  validateRequiredEnv();
  const github = new GitHubClient({ token: process.env.GITHUB_TOKEN });
  const commitDetailsBySha = {};

  const progress = (step, percentage) => onProgress({ step, percentage });
  const activity = await github.fetchDeveloperActivity({
    username: github_handle,
    repoFilter: repo_filter,
    onProgress: ({ step, percentage }) => progress(step, percentage),
  });

  const prAnalyses = [];
  const total = activity.prs.length || 1;
  for (let i = 0; i < activity.prs.length; i += 1) {
    const pr = activity.prs[i];
    progress("analysing_prs", Math.round(30 + ((i + 1) / total) * 45));
    const analysisRaw = await analysePR(pr, { anthropicApiKey: process.env.ANTHROPIC_API_KEY });
    const dimensions = {
      problem_solving_depth: Number(analysisRaw.problem_solving_depth || 0),
      code_quality: Number(analysisRaw.code_quality || 0),
      review_responsiveness: Number(
        analysisRaw.review_responsiveness == null ? 50 : analysisRaw.review_responsiveness
      ),
      scope_appropriateness: Number(analysisRaw.scope_appropriateness || 0),
      net_complexity_reduction: Number(analysisRaw.net_complexity_reduction || 0),
      gaming_likelihood: Number(analysisRaw.gaming_likelihood || 0),
    };
    const prScore = Number(
      analysisRaw.pr_score == null ? computePRScoreFromDimensions(dimensions) : analysisRaw.pr_score
    );
    const weight = Math.max(0, Math.min(1, Number(analysisRaw.weight_multiplier == null ? 1 : analysisRaw.weight_multiplier)));
    pr.analysis = {
      dimensions,
      pr_score: prScore,
      verdict: analysisRaw.verdict || "No verdict provided.",
      weight_multiplier: weight,
    };
    const analysisRecord = {
      repo: pr.repo,
      pr_number: pr.number,
      pr_title: pr.title,
      pr_url: pr.html_url,
      raw_additions: pr.additions,
      raw_deletions: pr.deletions,
      changed_files: pr.changed_files,
      review_rounds: (pr.reviews || []).length,
      dimensions,
      pr_score: prScore,
      weight_multiplier: weight,
      verdict: pr.analysis.verdict,
    };
    prAnalyses.push(analysisRecord);
  }

  for (const commit of activity.commits) {
    try {
      commitDetailsBySha[commit.sha] = await github.getCommitDetail(
        commit.repo.split("/")[0],
        commit.repo.split("/")[1],
        commit.sha
      );
    } catch (_) {
      commitDetailsBySha[commit.sha] = {};
    }
  }

  progress("detecting_gaming", 80);
  const antiCheat = applyAntiCheat({
    prs: activity.prs,
    commits: activity.commits,
    commitDetailsBySha,
  });

  const finalWeights = new Map(
    activity.prs.map((pr) => [`${pr.repo}#${pr.number}`, Number(pr.analysis?.weight_multiplier ?? 1)])
  );
  const adjustedPrAnalyses = prAnalyses.map((pr) => ({
    ...pr,
    weight_multiplier: finalWeights.get(`${pr.repo}#${pr.pr_number}`) ?? pr.weight_multiplier,
  }));

  progress("computing_score", 85);
  const scoreBreakdown = aggregateScore(adjustedPrAnalyses, activity.commits, activity.since);
  const period = `${new Date(activity.since).toISOString().slice(0, 10)}..${new Date(activity.until)
    .toISOString()
    .slice(0, 10)}`;

  const evidenceUnsigned = {
    schema_version: "1.0",
    generated_at: new Date().toISOString(),
    developer: {
      github_handle,
      wallet_address,
    },
    period: { start: activity.since, end: activity.until },
    raw_stats: {
      total_commits: activity.commits.length,
      total_prs_merged: activity.prs.length,
      raw_lines_added: activity.prs.reduce((sum, pr) => sum + Number(pr.additions || 0), 0),
      raw_lines_removed: activity.prs.reduce((sum, pr) => sum + Number(pr.deletions || 0), 0),
    },
    adjusted_stats: {
      weighted_commits: antiCheat.adjusted.weighted_commits,
      effective_lines_added: antiCheat.adjusted.effective_lines_added,
      inflation_removed_pct: antiCheat.adjusted.inflation_removed_pct,
    },
    gaming_flags: antiCheat.gaming_flags,
    pr_analyses: adjustedPrAnalyses,
    score_breakdown: scoreBreakdown,
    raw_github_payload: {
      repos: activity.repos,
      prs: activity.prs.map((pr) => pr.raw),
      commits: activity.commits,
      commit_details: commitDetailsBySha,
    },
  };

  const signedEvidence = await signEvidence(evidenceUnsigned, process.env.AGENT_PRIVATE_KEY);

  progress("writing_to_basin", 90);
  const cid = await writeEvidenceToBasin(signedEvidence, {
    basinApiUrl: process.env.BASIN_API_URL,
    bucket: process.env.BASIN_BUCKET,
  });

  progress("submitting_on_chain", 95);
  const chain = await submitOnChain({
    developer: wallet_address,
    github_handle,
    score: scoreBreakdown.final_score,
    tier: scoreBreakdown.tier,
    evidence_cid: cid,
    period,
    document_hash: signedEvidence.document_hash,
    agent_address: signedEvidence.agent_address,
    signature: signedEvidence.agent_signature,
  });

  const result = {
    github_handle,
    wallet_address,
    score: scoreBreakdown.final_score,
    tier: scoreBreakdown.tier,
    tier_code: scoreBreakdown.tier_code,
    evidence_cid: cid,
    score_breakdown: scoreBreakdown,
    pr_analyses: adjustedPrAnalyses,
    gaming_flags: antiCheat.gaming_flags,
    adjusted_stats: antiCheat.adjusted,
    on_chain_submission: chain,
    document_hash: signedEvidence.document_hash,
    agent_signature: signedEvidence.agent_signature,
    agent_address: signedEvidence.agent_address,
    updated_at: new Date().toISOString(),
  };

  progress("complete", 100);
  return result;
}

function createApp() {
  const app = express();
  app.use(express.json({ limit: "2mb" }));

  app.get("/health", (_req, res) => {
    res.status(200).json({
      ok: true,
      version: VERSION,
      authorised_agent_address: process.env.ADMIN_ADDRESS || null,
    });
  });

  app.post("/score", async (req, res) => {
    const github_handle = String(req.body?.github_handle || "").trim();
    const wallet_address = String(req.body?.wallet_address || "").trim();
    const repo_filter = req.body?.repo_filter;
    if (!github_handle || !wallet_address) {
      return res.status(400).json({ error: "github_handle and wallet_address are required" });
    }

    const job_id = crypto.randomUUID();
    const job = {
      job_id,
      status: "queued",
      progress: { step: "queued", percentage: 0 },
      created_at: new Date().toISOString(),
    };
    jobStore.set(job_id, job);

    setImmediate(async () => {
      job.status = "running";
      try {
        const result = await runPipeline({
          github_handle,
          wallet_address,
          repo_filter,
          onProgress: (progress) => {
            job.progress = progress;
          },
        });
        job.status = "complete";
        job.progress = { step: "complete", percentage: 100 };
        job.result = result;
        latestScores.set(github_handle.toLowerCase(), result);
      } catch (error) {
        job.status = "error";
        job.error = error.message;
      }
      job.updated_at = new Date().toISOString();
    });

    return res.status(202).json({ job_id });
  });

  app.get("/job/:job_id", (req, res) => {
    const job = jobStore.get(req.params.job_id);
    if (!job) {
      return res.status(404).json({ error: "job not found" });
    }
    return res.json(job);
  });

  app.get("/score/:github_handle", (req, res) => {
    const result = latestScores.get(String(req.params.github_handle || "").toLowerCase());
    if (!result) {
      return res.status(404).json({ error: "score not found for handle" });
    }
    return res.json(result);
  });

  return app;
}

function startServer() {
  const app = createApp();
  const port = Number(process.env.PORT || 3001);
  app.listen(port, () => {
    console.log(`dev-reputation agent listening on :${port}`);
  });
}

async function runCli() {
  const github_handle = process.argv[2];
  const wallet_address = process.argv[3];
  const repo_filter = process.argv[4];
  if (!github_handle || !wallet_address) {
    console.error("Usage: node src/index.js <github_handle> <wallet_address> [repo_filter]");
    process.exit(1);
  }
  const result = await runPipeline({
    github_handle,
    wallet_address,
    repo_filter,
    onProgress: (p) => console.log(`${p.step} ${p.percentage}%`),
  });
  console.log(JSON.stringify(result, null, 2));
}

if (require.main === module) {
  if (process.env.AGENT_MODE === "cli") {
    runCli().catch((error) => {
      console.error(error);
      process.exit(1);
    });
  } else {
    startServer();
  }
}

module.exports = {
  VERSION,
  aggregateScore,
  calculateConsistencyScore,
  computePRScoreFromDimensions,
  createApp,
  runPipeline,
  tierForScore,
};
