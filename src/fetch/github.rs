use chrono::{DateTime, Utc};
use color_eyre::{eyre::eyre, Result};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

/// GitHub API response for a repository
#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    pub stargazers_count: i64,
    pub open_issues_count: i64, // Note: includes PRs
    pub archived: bool,
    pub pushed_at: Option<String>,
    pub message: Option<String>, // Error message if request failed
}

/// GitHub API response for commits
#[derive(Debug, Deserialize)]
pub struct CommitInfo {
    pub commit: CommitDetail,
}

#[derive(Debug, Deserialize)]
pub struct CommitDetail {
    pub committer: CommitterInfo,
}

#[derive(Debug, Deserialize)]
pub struct CommitterInfo {
    pub date: String,
}

/// GitHub API response for pull requests (we just need count)
#[derive(Debug, Deserialize)]
pub struct PullRequest {
    pub id: i64,
}

/// GitHub API response for contributors
#[derive(Debug, Deserialize)]
pub struct Contributor {
    pub id: i64,
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
}

/// Metadata fetched from GitHub
#[derive(Debug, Clone)]
pub struct ProjectMetadata {
    pub stars: i64,
    pub open_issues: i64,
    pub open_prs: i64,
    pub contributors: i64,
    pub last_commit: Option<DateTime<Utc>>,
    pub is_archived: bool,
}

/// GitHub API client with rate limiting
pub struct GitHubClient {
    client: reqwest::Client,
    token: Option<String>,
    rate_limit: Option<RateLimit>,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(token: Option<String>) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            "application/vnd.github.v3+json".parse().unwrap(),
        );
        headers.insert(
            reqwest::header::USER_AGENT,
            "awesome-muncher/0.1".parse().unwrap(),
        );

        if let Some(ref t) = token {
            if let Ok(val) = format!("Bearer {}", t).parse() {
                headers.insert(reqwest::header::AUTHORIZATION, val);
            }
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            token,
            rate_limit: None,
        }
    }

    /// Check if we have a token configured
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    /// Get current rate limit status
    pub fn rate_limit(&self) -> Option<&RateLimit> {
        self.rate_limit.as_ref()
    }

    /// Update rate limit from response headers
    fn update_rate_limit(&mut self, headers: &reqwest::header::HeaderMap) {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let reset_timestamp = headers
            .get("x-ratelimit-reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let reset_at = DateTime::from_timestamp(reset_timestamp, 0)
            .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

        self.rate_limit = Some(RateLimit {
            remaining,
            reset_at,
        });
    }

    /// Wait if we're rate limited
    async fn check_rate_limit(&self) -> Result<()> {
        if let Some(ref limit) = self.rate_limit {
            if limit.remaining == 0 {
                let wait_time = (limit.reset_at - Utc::now()).num_seconds().max(0) as u64;
                if wait_time > 0 {
                    // Don't wait more than 5 minutes
                    let wait_time = wait_time.min(300);
                    sleep(Duration::from_secs(wait_time)).await;
                }
            }
        }
        Ok(())
    }

    /// Fetch repository information
    pub async fn get_repo_info(&mut self, owner: &str, repo: &str) -> Result<ProjectMetadata> {
        self.check_rate_limit().await?;

        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let response = self.client.get(&url).send().await?;

        self.update_rate_limit(response.headers());

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(eyre!(
                "GitHub API error for {}/{}: {} - {}",
                owner,
                repo,
                status,
                body
            ));
        }

        let repo_info: RepoInfo = response.json().await?;

        // Fetch additional data
        let last_commit = self.get_last_commit(owner, repo).await.ok().flatten();
        let open_prs = self.get_open_pr_count(owner, repo).await.unwrap_or(0);
        let contributors = self.get_contributor_count(owner, repo).await.unwrap_or(0);

        Ok(ProjectMetadata {
            stars: repo_info.stargazers_count,
            open_issues: repo_info.open_issues_count - open_prs, // Subtract PRs from issues
            open_prs,
            contributors,
            last_commit,
            is_archived: repo_info.archived,
        })
    }

    /// Get the date of the last commit
    async fn get_last_commit(
        &mut self,
        owner: &str,
        repo: &str,
    ) -> Result<Option<DateTime<Utc>>> {
        self.check_rate_limit().await?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?per_page=1",
            owner, repo
        );
        let response = self.client.get(&url).send().await?;

        self.update_rate_limit(response.headers());

        if !response.status().is_success() {
            return Ok(None);
        }

        let commits: Vec<CommitInfo> = response.json().await?;

        if let Some(commit) = commits.first() {
            if let Ok(date) = DateTime::parse_from_rfc3339(&commit.commit.committer.date) {
                return Ok(Some(date.with_timezone(&Utc)));
            }
        }

        Ok(None)
    }

    /// Get count of open pull requests
    async fn get_open_pr_count(&mut self, owner: &str, repo: &str) -> Result<i64> {
        self.check_rate_limit().await?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls?state=open&per_page=1",
            owner, repo
        );
        let response = self.client.get(&url).send().await?;

        self.update_rate_limit(response.headers());

        // Get count from Link header if available
        if let Some(link) = response.headers().get("link") {
            if let Ok(link_str) = link.to_str() {
                // Parse last page number from Link header
                if let Some(count) = parse_link_count(link_str) {
                    return Ok(count);
                }
            }
        }

        // Otherwise count the response
        if response.status().is_success() {
            let prs: Vec<PullRequest> = response.json().await?;
            return Ok(prs.len() as i64);
        }

        Ok(0)
    }

    /// Get count of contributors
    async fn get_contributor_count(&mut self, owner: &str, repo: &str) -> Result<i64> {
        self.check_rate_limit().await?;

        let url = format!(
            "https://api.github.com/repos/{}/{}/contributors?per_page=1&anon=true",
            owner, repo
        );
        let response = self.client.get(&url).send().await?;

        self.update_rate_limit(response.headers());

        // Get count from Link header if available
        if let Some(link) = response.headers().get("link") {
            if let Ok(link_str) = link.to_str() {
                if let Some(count) = parse_link_count(link_str) {
                    return Ok(count);
                }
            }
        }

        // Otherwise count the response
        if response.status().is_success() {
            let contributors: Vec<Contributor> = response.json().await?;
            return Ok(contributors.len() as i64);
        }

        Ok(0)
    }

    /// Check if we're currently rate limited
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limit
            .as_ref()
            .is_some_and(|limit| limit.remaining == 0)
    }

    /// Get time until rate limit resets
    pub fn rate_limit_reset_in(&self) -> Option<i64> {
        self.rate_limit.as_ref().map(|limit| {
            (limit.reset_at - Utc::now()).num_seconds().max(0)
        })
    }

    /// Fetch metadata for a project, returning None if it fails (e.g., repo doesn't exist)
    /// Returns (Option<metadata>, was_rate_limited)
    pub async fn fetch_project_metadata(
        &mut self,
        owner: &str,
        repo: &str,
    ) -> (Option<ProjectMetadata>, bool) {
        // Check if we're already rate limited before making request
        if self.is_rate_limited() {
            return (None, true);
        }

        match self.get_repo_info(owner, repo).await {
            Ok(metadata) => (Some(metadata), false),
            Err(_) => {
                // Check if the error was due to rate limiting
                let rate_limited = self.is_rate_limited();
                (None, rate_limited)
            }
        }
    }
}

/// Parse the last page number from a GitHub Link header
fn parse_link_count(link_header: &str) -> Option<i64> {
    // Link header format: <url>; rel="next", <url?page=X>; rel="last"
    for part in link_header.split(',') {
        if part.contains("rel=\"last\"") {
            // Extract page number from URL
            if let Some(start) = part.find("page=") {
                let after_page = &part[start + 5..];
                if let Some(end) = after_page.find(|c: char| !c.is_ascii_digit()) {
                    if let Ok(num) = after_page[..end].parse() {
                        return Some(num);
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_link_count() {
        let link = r#"<https://api.github.com/repos/a/b/pulls?page=2>; rel="next", <https://api.github.com/repos/a/b/pulls?page=15>; rel="last""#;
        assert_eq!(parse_link_count(link), Some(15));
    }

    #[test]
    fn test_parse_link_count_no_last() {
        let link = r#"<https://api.github.com/repos/a/b/pulls?page=2>; rel="next""#;
        assert_eq!(parse_link_count(link), None);
    }
}
