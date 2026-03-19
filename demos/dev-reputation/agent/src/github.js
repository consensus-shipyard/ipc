const axios = require("axios");

const GITHUB_API = "https://api.github.com";

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

class GitHubClient {
  constructor({ token } = {}) {
    this.http = axios.create({
      baseURL: GITHUB_API,
      timeout: 30000,
      headers: {
        Accept: "application/vnd.github+json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
      },
    });
  }

  async request(config, maxRetries = 5) {
    let attempt = 0;
    let backoffMs = 1000;
    while (attempt <= maxRetries) {
      try {
        const response = await this.http.request(config);
        return response.data;
      } catch (error) {
        const status = error.response?.status;
        const retryAfter = Number(error.response?.headers?.["retry-after"]);
        const isRateLimit = status === 403 || status === 429;
        const canRetry = attempt < maxRetries && (isRateLimit || !status || status >= 500);
        if (!canRetry) {
          throw error;
        }
        const waitMs = Number.isFinite(retryAfter) && retryAfter > 0 ? retryAfter * 1000 : backoffMs;
        await sleep(waitMs);
        backoffMs = Math.min(backoffMs * 2, 30000);
        attempt += 1;
      }
    }
    throw new Error("GitHub request exhausted retries");
  }

  async getUserRepos(username, limit = 5) {
    const repos = await this.request({
      method: "GET",
      url: `/users/${encodeURIComponent(username)}/repos`,
      params: { type: "all", sort: "pushed", per_page: Math.max(limit, 5) },
    });
    return repos.slice(0, limit);
  }

  async getMergedPRs(owner, repo, username, sinceIso) {
    const prs = await this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/pulls`,
      params: { state: "closed", creator: username, per_page: 100 },
    });
    return prs.filter((pr) => pr.merged_at && new Date(pr.merged_at) >= new Date(sinceIso));
  }

  async getPRFiles(owner, repo, prNumber) {
    return this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/pulls/${prNumber}/files`,
      params: { per_page: 100 },
    });
  }

  async getPRReviews(owner, repo, prNumber) {
    return this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/pulls/${prNumber}/reviews`,
      params: { per_page: 100 },
    });
  }

  async getPRComments(owner, repo, prNumber) {
    return this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/pulls/${prNumber}/comments`,
      params: { per_page: 100 },
    });
  }

  async getCommits(owner, repo, username, sinceIso) {
    return this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/commits`,
      params: { author: username, since: sinceIso, per_page: 100 },
    });
  }

  async getCommitDetail(owner, repo, sha) {
    return this.request({
      method: "GET",
      url: `/repos/${owner}/${repo}/commits/${sha}`,
    });
  }

  async fetchDeveloperActivity({
    username,
    repoFilter,
    days = 90,
    maxRepos = 5,
    onProgress = () => {},
  }) {
    const since = new Date(Date.now() - days * 24 * 60 * 60 * 1000).toISOString();

    onProgress({ step: "fetching_repos", percentage: 5 });
    let repos = await this.getUserRepos(username, maxRepos);
    if (repoFilter) {
      repos = repos.filter((repo) => `${repo.owner.login}/${repo.name}` === repoFilter || repo.name === repoFilter);
    }

    const prRecords = [];
    const commitRecords = [];

    onProgress({ step: "fetching_prs", percentage: 15 });
    for (const repo of repos) {
      const owner = repo.owner.login;
      const repoName = repo.name;
      let prs = [];
      let commits = [];
      try {
        prs = await this.getMergedPRs(owner, repoName, username, since);
      } catch (error) {
        if (error.response?.status !== 404) {
          throw error;
        }
      }
      try {
        commits = await this.getCommits(owner, repoName, username, since);
      } catch (error) {
        if (error.response?.status !== 404) {
          throw error;
        }
      }
      commitRecords.push(
        ...commits.map((commit) => ({
          repo: `${owner}/${repoName}`,
          ...commit,
        }))
      );
      prRecords.push(
        ...prs.map((pr) => ({
          repo: `${owner}/${repoName}`,
          owner,
          repoName,
          ...pr,
        }))
      );
    }

    onProgress({ step: "fetching_diffs", percentage: 30 });
    for (let i = 0; i < prRecords.length; i += 1) {
      const pr = prRecords[i];
      const [files, reviews, comments] = await Promise.all([
        this.getPRFiles(pr.owner, pr.repoName, pr.number),
        this.getPRReviews(pr.owner, pr.repoName, pr.number),
        this.getPRComments(pr.owner, pr.repoName, pr.number),
      ]);
      pr.files = files;
      pr.reviews = reviews;
      pr.comments = comments;
      pr.raw = { pr, files, reviews, comments };
    }

    return {
      since,
      until: new Date().toISOString(),
      repos,
      prs: prRecords,
      commits: commitRecords,
    };
  }
}

module.exports = {
  GitHubClient,
};
