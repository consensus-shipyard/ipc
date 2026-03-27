const test = require("node:test");
const assert = require("node:assert/strict");
const { aggregateScore } = require("../src/index");

test("aggregateScore follows weighted formula", () => {
  const prs = [
    {
      pr_score: 90,
      weight_multiplier: 1.0,
      dimensions: {
        problem_solving_depth: 90,
        code_quality: 85,
        review_responsiveness: 80,
        scope_appropriateness: 70,
        net_complexity_reduction: 75,
      },
    },
    {
      pr_score: 50,
      weight_multiplier: 0.5,
      dimensions: {
        problem_solving_depth: 45,
        code_quality: 55,
        review_responsiveness: 60,
        scope_appropriateness: 50,
        net_complexity_reduction: 40,
      },
    },
  ];

  const commits = Array.from({ length: 12 }, (_, i) => ({
    commit: { author: { date: new Date(Date.now() - i * 7 * 24 * 60 * 60 * 1000).toISOString() } },
  }));

  const since = new Date(Date.now() - 90 * 24 * 60 * 60 * 1000).toISOString();
  const result = aggregateScore(prs, commits, since);

  const weightedPr = (90 * 1 + 50 * 0.5) / 1.5;
  assert.equal(result.weighted_pr_score, Number(weightedPr.toFixed(2)));
  assert.equal(result.final_score >= 0 && result.final_score <= 100, true);
  assert.equal(typeof result.tier, "string");
});
