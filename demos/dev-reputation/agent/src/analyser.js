const axios = require("axios");

const ANALYSIS_SYSTEM_PROMPT = `You are a senior engineering evaluator scoring pull requests for technical merit.
You evaluate the ACTUAL WORK done, not surface metrics like line count.

You will be given a pull request diff along with its title, description,
file list, and review comment thread. Evaluate it on the following six
dimensions and return a JSON object — nothing else, no prose, no markdown.

Dimensions:
  problem_solving_depth (0-100):
    How hard was the underlying problem? Does the solution show real
    understanding of the domain? Penalise obvious or mechanical solutions.

  code_quality (0-100):
    Is the code readable, well-structured, and appropriately tested?
    Does it handle edge cases? Is complexity justified?

  review_responsiveness (0-100):
    How well did the author engage with review feedback? Did they
    explain their reasoning? Did they push back thoughtfully when appropriate?
    Score 50 if there were no review comments.

  scope_appropriateness (0-100):
    Is the PR an appropriate unit of work — not too large, not trivially
    small? Does it solve exactly one thing cleanly?

  net_complexity_reduction (0-100):
    Did the PR make the codebase simpler or harder to understand?
    Refactors that reduce coupling score high. Features that add
    necessary complexity score medium. Pure additions score lower.

  gaming_likelihood (0-100):
    How likely is this PR to be padding rather than real work?
    0 = clearly genuine, 100 = almost certainly gaming.
    Look for: moved-without-changed code, formatter-only commits,
    generated output committed verbatim, trivial renames at scale.

Also return:
  pr_score (0-100): weighted aggregate of the above dimensions
  verdict (string): 2-3 sentence plain English summary of what
    the PR demonstrates about the developer. Be specific about
    the technical content — name the pattern, technique, or
    problem domain. Do not be generic.
  weight_multiplier (float 0.0-1.0): how much this PR should
    contribute to the overall score. 1.0 for substantive work,
    0.5 for medium-value PRs, 0.1 for near-trivial, 0.0 for
    pure gaming. This is your judgment call.

Return only valid JSON. No prose before or after.`;

function summariseReviewThread(reviews = [], comments = []) {
  if (!reviews.length && !comments.length) {
    return "No review comments.";
  }
  const reviewSummary = reviews
    .slice(0, 8)
    .map((r) => `${r.user?.login || "reviewer"}:${r.state || "COMMENTED"}:${(r.body || "").slice(0, 160)}`)
    .join(" | ");
  const commentSummary = comments
    .slice(0, 10)
    .map((c) => `${c.user?.login || "reviewer"}:${(c.body || "").slice(0, 160)}`)
    .join(" | ");
  return [reviewSummary, commentSummary].filter(Boolean).join(" || ");
}

function truncatePatch(patch = "", perFileLimit = 6000) {
  if (!patch) {
    return "";
  }
  if (patch.length <= perFileLimit) {
    return patch;
  }
  return `${patch.slice(0, 3000)}\n\n... [TRUNCATED] ...\n\n${patch.slice(-3000)}`;
}

function buildDiffText(files = [], globalLimit = 32000) {
  const joined = files
    .map((f) => `FILE: ${f.filename}\nSTATUS: ${f.status}\n${truncatePatch(f.patch || "")}`)
    .join("\n\n");
  if (joined.length <= globalLimit) {
    return joined;
  }
  return `${joined.slice(0, globalLimit / 2)}\n\n... [GLOBAL TRUNCATION] ...\n\n${joined.slice(-globalLimit / 2)}`;
}

function safeJsonParse(text) {
  const trimmed = text.trim();
  try {
    return JSON.parse(trimmed);
  } catch (_) {
    const firstBrace = trimmed.indexOf("{");
    const lastBrace = trimmed.lastIndexOf("}");
    if (firstBrace >= 0 && lastBrace > firstBrace) {
      return JSON.parse(trimmed.slice(firstBrace, lastBrace + 1));
    }
    throw new Error("Claude response was not valid JSON");
  }
}

async function analysePR(pr, { anthropicApiKey }) {
  const reviewSummary = summariseReviewThread(pr.reviews, pr.comments);
  const diff = buildDiffText(pr.files);
  const userPrompt = `PR title: ${pr.title}
PR description: ${pr.body || "none"}
Files changed: ${pr.changed_files}, Additions: ${pr.additions}, Deletions: ${pr.deletions}
Review rounds: ${(pr.reviews || []).length}
Review comments summary: ${reviewSummary}

Full diff (truncated to 8000 tokens if necessary):
${diff}`;

  const response = await axios.post(
    "https://api.anthropic.com/v1/messages",
    {
      model: "claude-sonnet-4-20250514",
      max_tokens: 1200,
      temperature: 0.1,
      system: ANALYSIS_SYSTEM_PROMPT,
      messages: [{ role: "user", content: userPrompt }],
    },
    {
      headers: {
        "Content-Type": "application/json",
        "x-api-key": anthropicApiKey,
        "anthropic-version": "2023-06-01",
      },
      timeout: 60000,
    }
  );

  const text = response.data?.content?.[0]?.text || "{}";
  return safeJsonParse(text);
}

module.exports = {
  ANALYSIS_SYSTEM_PROMPT,
  analysePR,
  buildDiffText,
  summariseReviewThread,
};
